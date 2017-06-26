// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::*;
use misc;
use ratio;
use midi;
use jack;


struct SharedData {
	events: Vec<midi::Event>,
	changed: bool,
}

// XXX: unmovable mark or 2nd depth indirection.
pub struct Player {
	shared: sync::Mutex<SharedData>,
	jack: *mut jack::Client,
	port: *mut jack::Port,
}

unsafe impl Send for Player {}

impl Drop for Player {
	fn drop( &mut self ) {
		unsafe {
			jack::jack_client_close( self.jack );
		}
	}
}

impl Player {
	pub fn new( name: &str ) -> io::Result<Box<Player>> {
		unsafe {
			let jack = jack::jack_client_open( c_str!( "{}", name ), 0, ptr::null_mut() );
			if jack.is_null() {
				return Err( io::Error::new( io::ErrorKind::Other, "jack_client_open()." ) );
			}

			let port = jack::jack_port_register( jack, c_str!( "out" ), c_str!( "8 bit raw midi" ), jack::PORT_IS_OUTPUT, 0 );
			if port.is_null() {
				jack::jack_client_close( jack );
				return Err( io::Error::new( io::ErrorKind::Other, "jack_port_register()." ) );
			}

			let mut this = Box::new( Player{
				shared: sync::Mutex::new( SharedData{
					events: Vec::new(),
					changed: false,
				} ),
				jack: jack,
				port: port,
			} );

			if jack::jack_set_process_callback( this.jack, Player::process_callback, &mut *this ) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "jack_set_process_callback()." ) );
			}
			if jack::jack_set_sync_callback( this.jack, Player::sync_callback, &mut *this ) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "jack_set_sync_callback()." ) );
			}

			if jack::jack_activate( this.jack ) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "jack_activate()." ) );
			}

			Ok( this )
		}
	}

	pub fn set_data( &self, events: Vec<midi::Event> ) {
		let mut shared = self.shared.lock().unwrap();
		shared.events = events;
		shared.changed = true;
	}

	pub fn connect( &self, port: &str ) -> io::Result<()> {
		unsafe {
			if jack::jack_connect( self.jack, jack::jack_port_name( self.port ), c_str!( "{}", port ) ) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "jack_connect()." ) );
			}
		}
		Ok( () )
	}

	pub fn play( &self ) -> io::Result<()> {
		unsafe {
			jack::jack_transport_start( self.jack );
		}
		Ok( () )
	}

	pub fn stop( &self ) -> io::Result<()> {
		unsafe {
			jack::jack_transport_stop( self.jack );
		}
		let mut shared = self.shared.lock().unwrap();
		shared.changed = true;
		Ok( () )
	}

	pub fn seek( &self, time: f64 ) -> io::Result<()> {
		unsafe {
			let mut pos: jack::Position = mem::uninitialized();
			jack::jack_transport_query( self.jack, &mut pos );
			jack::jack_transport_locate( self.jack, (time * pos.frame_rate as f64) as u32 );
		}
		Ok( () )
	}

	pub fn location( &self ) -> ratio::Ratio {
		unsafe {
			let mut pos: jack::Position = mem::uninitialized();
			jack::jack_transport_query( self.jack, &mut pos );
			// the resolution of jack_position_t::frame is per process cycles.
			// jack_get_current_transport_frame() estimates the current
			// position more accurately.
			let frame = jack::jack_get_current_transport_frame( self.jack );
			ratio::Ratio::new( frame as i64, pos.frame_rate as i64 )
		}
	}

	pub fn is_playing( &self ) -> bool {
		unsafe {
			let mut pos: jack::Position = mem::uninitialized();
			match jack::jack_transport_query( self.jack, &mut pos ) {
				jack::TransportState::Stopped => false,
				_                             => true,
			}
		}
	}

	extern fn process_callback( size: u32, this: *mut any::Any ) -> i32 {
		unsafe {
			let this = &mut *(this as *mut Player);

			let buf = jack::jack_port_get_buffer( this.port, size );
			jack::jack_midi_clear_buffer( buf );

			let mut shared = match this.shared.try_lock() {
				Err( _ ) => return 0,
				Ok ( v ) => v,
			};

			if shared.changed {
				Self::write_all_note_off( buf, 0 );
				shared.changed = false;
			}

			let mut pos: jack::Position = mem::uninitialized();
			match jack::jack_transport_query( this.jack, &mut pos ) {
				jack::TransportState::Rolling => (),
				_ => return 0,
			}

			let time = |ev: &midi::Event| (ev.time * pos.frame_rate as f64).round() as u32;
			let ibgn = misc::bsearch_boundary( &shared.events, |ev| (time( ev ), ev.prio) < (pos.frame,        i32::MIN) );
			let iend = misc::bsearch_boundary( &shared.events, |ev| (time( ev ), ev.prio) < (pos.frame + size, i32::MIN) );
			for ev in shared.events[ibgn .. iend].iter() {
				jack::jack_midi_event_write( buf, time( ev ) - pos.frame, ev.msg.as_ptr(), ev.len as usize );
			}

			if ibgn == shared.events.len() {
				jack::jack_transport_stop( this.jack );
				shared.changed = true;
			}
		}
		0
	}

	unsafe fn write_all_note_off( buf: *mut jack::PortBuffer, frame: u32 ) {
		for ch in 0 .. 16 {
			let msg: [u8; 3] = [ 0xb0 + ch, 0x7b, 0x00 ];
			jack::jack_midi_event_write( buf, frame, msg.as_ptr(), msg.len() );
		}
	}

	extern fn sync_callback( _: jack::TransportState, _: *mut jack::Position, this: *mut any::Any ) -> i32 {
		unsafe {
			let this = &mut *(this as *mut Player);

			let mut shared = match this.shared.try_lock() {
				Err( _ ) => return 0,
				Ok ( v ) => v,
			};

			shared.changed = true;
			1 // ready to roll.
		}
	}
}
