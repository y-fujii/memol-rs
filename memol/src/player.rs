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
	lib: jack::Library,
	jack: *mut jack::Client,
	port: *mut jack::Port,
	shared: sync::Mutex<SharedData>,
}

unsafe impl Send for Player {}

impl Drop for Player {
	fn drop( &mut self ) {
		unsafe {
			(self.lib.client_close)( self.jack );
		}
	}
}

impl Player {
	pub fn new( name: &str ) -> io::Result<Box<Player>> {
		unsafe {
			let lib = jack::Library::new()?;

			let jack = (lib.client_open)( format!( "{}\0", name ).as_ptr(), 0, ptr::null_mut() );
			if jack.is_null() {
				return Err( io::Error::new( io::ErrorKind::Other, "jack_client_open()." ) );
			}

			let port = (lib.port_register)( jack, "out\0".as_ptr(), jack::DEFAULT_MIDI_TYPE, jack::PORT_IS_OUTPUT, 0 );
			if port.is_null() {
				(lib.client_close)( jack );
				return Err( io::Error::new( io::ErrorKind::Other, "jack_port_register()." ) );
			}

			let this = Box::new( Player{
				lib: lib,
				jack: jack,
				port: port,
				shared: sync::Mutex::new( SharedData{
					events: Vec::new(),
					changed: false,
				} ),
			} );

			if (this.lib.set_process_callback)( this.jack, Player::process_callback, &*this ) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "jack_set_process_callback()." ) );
			}
			if (this.lib.set_sync_callback)( this.jack, Player::sync_callback, &*this ) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "jack_set_sync_callback()." ) );
			}

			if (this.lib.activate)( this.jack ) != 0 {
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

	pub fn ports( &self ) -> io::Result<Vec<(String, bool)>> {
		unsafe {
			let mut c_result = (self.lib.get_ports)( self.jack, ptr::null(), jack::DEFAULT_MIDI_TYPE, jack::PORT_IS_INPUT );
			if c_result.is_null() {
				return Err( io::Error::new( io::ErrorKind::Other, "jack_get_ports()." ) );
			}
			let mut r_result = Vec::new();
			while !(*c_result).is_null() {
				match ffi::CStr::from_ptr( *c_result as *const _ ).to_str() {
					Ok( v ) => {
						let is_conn = (self.lib.port_connected_to)( self.port, format!( "{}\0", v ).as_ptr() );
						r_result.push( (v.into(), is_conn != 0) );
					},
					Err( _ ) => {
						(self.lib.free)( c_result );
						return Err( io::Error::new( io::ErrorKind::Other, "jack_get_ports()." ) );
					},
				}
				c_result = c_result.offset( 1 );
			}
			(self.lib.free)( c_result );
			Ok( r_result )
		}
	}

	pub fn connect( &self, port: &str ) -> io::Result<()> {
		unsafe {
			if (self.lib.connect)( self.jack, (self.lib.port_name)( self.port ), format!( "{}\0", port ).as_ptr() ) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "jack_connect()." ) );
			}
		}
		Ok( () )
	}

	pub fn disconnect( &self, port: &str ) -> io::Result<()> {
		unsafe {
			if (self.lib.disconnect)( self.jack, (self.lib.port_name)( self.port ), format!( "{}\0", port ).as_ptr() ) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "jack_disconnect()." ) );
			}
		}
		Ok( () )
	}

	pub fn play( &self ) -> io::Result<()> {
		unsafe {
			(self.lib.transport_start)( self.jack );
		}
		Ok( () )
	}

	pub fn stop( &self ) -> io::Result<()> {
		unsafe {
			(self.lib.transport_stop)( self.jack );
		}
		let mut shared = self.shared.lock().unwrap();
		shared.changed = true;
		Ok( () )
	}

	pub fn seek( &self, time: f64 ) -> io::Result<()> {
		debug_assert!( time >= 0.0 );
		unsafe {
			let mut pos: jack::Position = mem::uninitialized();
			(self.lib.transport_query)( self.jack, &mut pos );
			(self.lib.transport_locate)( self.jack, (time * pos.frame_rate as f64) as u32 );
		}
		Ok( () )
	}

	pub fn location( &self ) -> ratio::Ratio {
		unsafe {
			let mut pos: jack::Position = mem::uninitialized();
			(self.lib.transport_query)( self.jack, &mut pos );
			// the resolution of jack_position_t::frame is per process cycles.
			// jack_get_current_transport_frame() estimates the current
			// position more accurately.
			let frame = (self.lib.get_current_transport_frame)( self.jack );
			ratio::Ratio::new( frame as i64, pos.frame_rate as i64 )
		}
	}

	pub fn is_playing( &self ) -> bool {
		unsafe {
			let mut pos: jack::Position = mem::uninitialized();
			match (self.lib.transport_query)( self.jack, &mut pos ) {
				jack::TransportState::Stopped => false,
				_                             => true,
			}
		}
	}

	extern fn process_callback( size: u32, this: *const any::Any ) -> i32 {
		unsafe {
			let this = &*(this as *const Player);

			let buf = (this.lib.port_get_buffer)( this.port, size );
			(this.lib.midi_clear_buffer)( buf );

			let mut shared = match this.shared.try_lock() {
				Err( _ ) => return 0,
				Ok ( v ) => v,
			};

			if shared.changed {
				this.write_all_note_off( buf, 0 );
				shared.changed = false;
			}

			let mut pos: jack::Position = mem::uninitialized();
			match (this.lib.transport_query)( this.jack, &mut pos ) {
				jack::TransportState::Rolling => (),
				_ => return 0,
			}

			let frame = |ev: &midi::Event| (ev.time * pos.frame_rate as f64 - pos.frame as f64).round();
			let ibgn = misc::bsearch_boundary( &shared.events, |ev| (frame( ev ), ev.prio) < (0.0  as f64, i16::MIN) );
			let iend = misc::bsearch_boundary( &shared.events, |ev| (frame( ev ), ev.prio) < (size as f64, i16::MIN) );
			for ev in shared.events[ibgn .. iend].iter() {
				(this.lib.midi_event_write)( buf, frame( ev ) as u32, ev.msg.as_ptr(), ev.len as usize );
			}

			if ibgn == shared.events.len() {
				(this.lib.transport_stop)( this.jack );
				shared.changed = true;
			}
		}
		0
	}

	unsafe fn write_all_note_off( &self, buf: *mut jack::PortBuffer, frame: u32 ) {
		for ch in 0 .. 16 {
			let msg: [u8; 3] = [ 0xb0 + ch, 0x7b, 0x00 ];
			(self.lib.midi_event_write)( buf, frame, msg.as_ptr(), msg.len() );
		}
	}

	extern fn sync_callback( _: jack::TransportState, _: *mut jack::Position, this: *const any::Any ) -> i32 {
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
}
