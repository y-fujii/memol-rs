// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use memol::misc;
use memol::midi;
use player;
use jack;


const BUFFER_LEN: usize = 65536 / 16; // mem::size_of<MidiEvent>() == 16.

struct SharedData {
	events: Vec<midi::Event>,
	changed: bool,
}

// XXX: unmovable mark or 2nd depth indirection.
pub struct Player {
	lib: jack::Library,
	jack: *mut jack::Client,
	port_send: *mut jack::Port,
	port_recv: *mut jack::Port,
	immediate_send: *mut jack::RingBuffer,
	immediate_recv: *mut jack::RingBuffer,
	shared: sync::Mutex<SharedData>,
	cb_thread: Option<thread::JoinHandle<()>>,
	cb_shared: sync::Arc<(sync::Mutex<bool>, sync::Condvar)>,
}

unsafe impl Send for Player {}

impl Drop for Player {
	fn drop( &mut self ) {
		if let Some( cb_thread ) = mem::replace( &mut self.cb_thread, None ) {
			*self.cb_shared.0.lock().unwrap() = true;
			self.cb_shared.1.notify_all();
			cb_thread.join().unwrap();
		}
		unsafe {
			(self.lib.client_close)( self.jack );
			(self.lib.ringbuffer_free)( self.immediate_recv );
			(self.lib.ringbuffer_free)( self.immediate_send );
		}
	}
}

impl player::Player for Player {
	fn on_received_boxed( &mut self, f: Box<'static + Fn() + Send> ) {
		if let Some( cb_thread ) = mem::replace( &mut self.cb_thread, None ) {
			cb_thread.join().unwrap();
		}
		self.cb_thread = Some( thread::spawn( {
			let cb_shared = self.cb_shared.clone();
			move || Self::cb_proc( f, cb_shared )
		} ) );
	}

	fn set_data( &self, events: Vec<midi::Event> ) {
		let mut shared = self.shared.lock().unwrap();
		shared.events = events;
		shared.changed = true;
	}

	fn ports_from( &self ) -> io::Result<Vec<(String, bool)>> {
		self.ports( jack::PORT_IS_OUTPUT )
	}

	fn connect_from( &self, port: &str ) -> io::Result<()> {
		unsafe {
			self.connect( format!( "{}\0", port ).as_ptr(), (self.lib.port_name)( self.port_recv ) )
		}
	}

	fn disconnect_from( &self, port: &str ) -> io::Result<()> {
		unsafe {
			self.disconnect( format!( "{}\0", port ).as_ptr(), (self.lib.port_name)( self.port_recv ) )
		}
	}

	fn ports_to( &self ) -> io::Result<Vec<(String, bool)>> {
		self.ports( jack::PORT_IS_INPUT )
	}

	fn connect_to( &self, port: &str ) -> io::Result<()> {
		unsafe {
			self.connect( (self.lib.port_name)( self.port_send ), format!( "{}\0", port ).as_ptr() )
		}
	}

	fn disconnect_to( &self, port: &str ) -> io::Result<()> {
		unsafe {
			self.disconnect( (self.lib.port_name)( self.port_send ), format!( "{}\0", port ).as_ptr() )
		}
	}

	fn send( &self, evs: &[midi::Event] ) -> io::Result<()> {
		unsafe {
			let mut i = 0;
			while i < evs.len() {
				i += self.rb_write_block( self.immediate_send, &evs[i ..] );
			}
		}
		Ok( () )
	}

	fn recv( &self, evs: &mut Vec<midi::Event> ) -> io::Result<()> {
		unsafe {
			let mut buf: [midi::Event; BUFFER_LEN] = mem::uninitialized();
			let len = self.rb_read_block( self.immediate_recv, &mut buf );
			evs.extend_from_slice( &buf[0 .. len] );
		}
		Ok( () )
	}

	fn play( &self ) -> io::Result<()> {
		unsafe {
			(self.lib.transport_start)( self.jack );
		}
		Ok( () )
	}

	fn stop( &self ) -> io::Result<()> {
		unsafe {
			(self.lib.transport_stop)( self.jack );
		}
		let mut shared = self.shared.lock().unwrap();
		shared.changed = true;
		Ok( () )
	}

	fn seek( &self, time: f64 ) -> io::Result<()> {
		debug_assert!( time >= 0.0 );
		unsafe {
			let mut pos: jack::Position = mem::uninitialized();
			(self.lib.transport_query)( self.jack, &mut pos );
			(self.lib.transport_locate)( self.jack, (time * pos.frame_rate as f64).round() as u32 );
		}
		Ok( () )
	}

	fn location( &self ) -> f64 {
		unsafe {
			let mut pos: jack::Position = mem::uninitialized();
			(self.lib.transport_query)( self.jack, &mut pos );
			// the resolution of jack_position_t::frame is per process cycles.
			// jack_get_current_transport_frame() estimates the current
			// position more accurately.
			let frame = (self.lib.get_current_transport_frame)( self.jack );
			frame as f64 / pos.frame_rate as f64
		}
	}

	fn is_playing( &self ) -> bool {
		unsafe {
			let mut pos: jack::Position = mem::uninitialized();
			match (self.lib.transport_query)( self.jack, &mut pos ) {
				jack::TransportState::Stopped => false,
				_                             => true,
			}
		}
	}
}

impl Player {
	pub fn new( name: &str ) -> io::Result<Box<Player>> {
		unsafe {
			let lib = jack::Library::new()?;

			let jack = (lib.client_open)( format!( "{}\0", name ).as_ptr(), 0, ptr::null_mut() );
			if jack.is_null() {
				return Self::error( "jack_client_open()." );
			}
			let port_send = (lib.port_register)( jack, "out\0".as_ptr(), jack::DEFAULT_MIDI_TYPE, jack::PORT_IS_OUTPUT, 0 );
			if port_send.is_null() {
				(lib.client_close)( jack );
				return Self::error( "jack_port_register()." );
			}
			let port_recv = (lib.port_register)( jack, "in\0".as_ptr(), jack::DEFAULT_MIDI_TYPE, jack::PORT_IS_INPUT, 0 );
			if port_recv.is_null() {
				(lib.client_close)( jack );
				return Self::error( "jack_port_register()." );
			}

			let immediate_send = (lib.ringbuffer_create)( mem::size_of::<midi::Event>() * BUFFER_LEN );
			if immediate_send.is_null() {
				(lib.client_close)( jack );
				return Self::error( "jack_ringbuffer_create()." );
			}
			let immediate_recv = (lib.ringbuffer_create)( mem::size_of::<midi::Event>() * BUFFER_LEN );
			if immediate_recv.is_null() {
				(lib.ringbuffer_free)( immediate_send );
				(lib.client_close)( jack );
				return Self::error( "jack_ringbuffer_create()." );
			}

			let this = Box::new( Player{
				lib: lib,
				jack: jack,
				port_send: port_send,
				port_recv: port_recv,
				immediate_send: immediate_send,
				immediate_recv: immediate_recv,
				shared: sync::Mutex::new( SharedData{
					events: Vec::new(),
					changed: false,
				} ),
				cb_shared: sync::Arc::new( (sync::Mutex::new( false ), sync::Condvar::new()) ),
				cb_thread: None,
			} );

			if (this.lib.set_process_callback)( this.jack, Player::process_callback, &*this ) != 0 {
				return Self::error( "jack_set_process_callback()." );
			}
			if (this.lib.set_sync_callback)( this.jack, Player::sync_callback, &*this ) != 0 {
				return Self::error( "jack_set_sync_callback()." );
			}

			if (this.lib.activate)( this.jack ) != 0 {
				return Self::error( "jack_activate()." );
			}

			Ok( this )
		}
	}

	fn ports( &self, port_type: usize ) -> io::Result<Vec<(String, bool)>> {
		unsafe {
			let self_port = match port_type {
				jack::PORT_IS_INPUT  => self.port_send,
				jack::PORT_IS_OUTPUT => self.port_recv,
				_                    => panic!(),
			};

			let c_result = (self.lib.get_ports)( self.jack, ptr::null(), jack::DEFAULT_MIDI_TYPE, port_type );
			if c_result.is_null() {
				return Self::error( "jack_get_ports()." );
			}
			let mut r_result = Vec::new();
			let mut it = c_result;
			while !(*it).is_null() {
				match ffi::CStr::from_ptr( *it as *const _ ).to_str() {
					Ok( v ) => {
						let is_conn = (self.lib.port_connected_to)( self_port, *it );
						r_result.push( (v.into(), is_conn != 0) );
					},
					Err( _ ) => {
						(self.lib.free)( c_result );
						return Self::error( "jack_get_ports()." );
					},
				}
				it = it.offset( 1 );
			}
			(self.lib.free)( c_result );
			Ok( r_result )
		}
	}

	unsafe fn connect( &self, from: *const u8, to: *const u8 ) -> io::Result<()> {
		if (self.lib.connect)( self.jack, from, to ) != 0 {
			return Self::error( "jack_connect()." );
		}
		Ok( () )
	}

	unsafe fn disconnect( &self, from: *const u8, to: *const u8 ) -> io::Result<()> {
		if (self.lib.disconnect)( self.jack, from, to ) != 0 {
			return Self::error( "jack_disconnect()." );
		}
		Ok( () )
	}

	fn error<T>( text: &str ) -> io::Result<T> {
		Err( io::Error::new( io::ErrorKind::Other, text ) )
	}

	unsafe fn rb_write_block<T>( &self, rb: *mut jack::RingBuffer, buf: &[T] ) -> usize {
		let len = cmp::min( (self.lib.ringbuffer_write_space)( rb ) / mem::size_of::<T>(), buf.len() );
		(self.lib.ringbuffer_write)( rb, buf.as_ptr() as *const u8, len * mem::size_of::<T>() );
		len
	}

	unsafe fn rb_read_block<T>( &self, rb: *mut jack::RingBuffer, buf: &mut [T] ) -> usize {
		let len = cmp::min( (self.lib.ringbuffer_read_space)( rb ) / mem::size_of::<T>(), buf.len() );
		(self.lib.ringbuffer_read)( rb, buf.as_mut_ptr() as *mut u8, len * mem::size_of::<T>() );
		len
	}

	extern "C" fn process_callback( size: u32, this: *const any::Any ) -> i32 {
		unsafe {
			let this = &*(this as *const Player);

			let buf_send = (this.lib.port_get_buffer)( this.port_send, size );
			let buf_recv = (this.lib.port_get_buffer)( this.port_recv, size );
			(this.lib.midi_clear_buffer)( buf_send );

			let mut pos: jack::Position = mem::uninitialized();
			let state = (this.lib.transport_query)( this.jack, &mut pos );

			for i in 0 .. {
				let mut ev = mem::uninitialized();
				if (this.lib.midi_event_get)( &mut ev, buf_recv, i ) != 0 {
					break;
				}
				let msg = slice::from_raw_parts( ev.buffer, ev.size );
				this.rb_write_block( this.immediate_recv, &[ midi::Event::new( ev.time as f64 / pos.frame_rate as f64, 0, msg ) ] );
			}
			// since we use the condition variable without a locked flag,
			// notification possibly fails.  we check the buffer every time.
			if (this.lib.ringbuffer_read_space)( this.immediate_recv ) >= mem::size_of::<midi::Event>() {
				this.cb_shared.1.notify_one();
			}

			let mut evs: [midi::Event; BUFFER_LEN] = mem::uninitialized();
			let len = this.rb_read_block( this.immediate_send, &mut evs );
			for ev in evs[0 .. len].iter() {
				(this.lib.midi_event_write)( buf_send, 0, ev.msg.as_ptr(), ev.len as usize );
			}

			let mut shared = match this.shared.try_lock() {
				Err( _ ) => return 0,
				Ok ( v ) => v,
			};

			if shared.changed {
				this.write_all_sound_off( buf_send, 0 );
				shared.changed = false;
			}

			if let jack::TransportState::Rolling = state {
				let frame = |ev: &midi::Event| (ev.time * pos.frame_rate as f64).round() as isize - pos.frame as isize;
				let ibgn = misc::bsearch_boundary( &shared.events, |ev| frame( ev ) < 0 );
				let iend = misc::bsearch_boundary( &shared.events, |ev| frame( ev ) < size as isize );
				for ev in shared.events[ibgn .. iend].iter() {
					(this.lib.midi_event_write)( buf_send, frame( ev ) as u32, ev.msg.as_ptr(), ev.len as usize );
				}

				if ibgn == shared.events.len() {
					(this.lib.transport_stop)( this.jack );
					shared.changed = true;
				}
			}
		}
		0
	}

	unsafe fn write_all_sound_off( &self, buf: *mut jack::PortBuffer, frame: u32 ) {
		for ch in 0 .. 16 {
			let msg: [u8; 3] = [ 0xb0 + ch, 0x78, 0x00 ]; // all sound off.
			(self.lib.midi_event_write)( buf, frame, msg.as_ptr(), msg.len() );
			let msg: [u8; 3] = [ 0xb0 + ch, 0x79, 0x00 ]; // reset all controllers.
			(self.lib.midi_event_write)( buf, frame, msg.as_ptr(), msg.len() );
		}
	}

	extern "C" fn sync_callback( _: jack::TransportState, _: *mut jack::Position, this: *const any::Any ) -> i32 {
		unsafe {
			let this = &*(this as *const Player);

			let mut shared = match this.shared.try_lock() {
				Err( _ ) => return 0,
				Ok ( v ) => v,
			};

			shared.changed = true;
			1 // ready to roll.
		}
	}

	fn cb_proc( on_received: Box<'static + Fn() + Send>, shared: sync::Arc<(sync::Mutex<bool>, sync::Condvar)> ) {
		let mut guard = shared.0.lock().unwrap();
		loop {
			guard = shared.1.wait( guard ).unwrap();
			if *guard {
				break;
			}
			// XXX: see the comment in process_callback().
			//if (this.lib.ringbuffer_read_space)( this.immediate_recv ) >= mem::size_of::<midi::Event>() {
				on_received();
			//}
		}
	}
}
