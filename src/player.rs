// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD license.
use std::*;
use jack;
use misc;
use ratio;
use midi;


struct SharedData {
	events: Vec<midi::Event>,
	changed: bool,
}

// XXX: unmovable mark or 2nd depth indirection.
pub struct Player {
	shared: sync::Mutex<SharedData>,
	jack: *mut jack::Client,
	port: *mut jack::Port,
	name: String,
}

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
			let jack = jack::jack_client_open(
				ffi::CString::new( name ).unwrap().as_ptr(),
				0,
				ptr::null_mut()
			);
			if jack.is_null() {
				return Err( io::Error::new( io::ErrorKind::Other, "" ) );
			}

			let port = jack::jack_port_register(
				jack,
				ffi::CString::new( "out" ).unwrap().as_ptr(),
				ffi::CString::new( "8 bit raw midi" ).unwrap().as_ptr(),
				jack::PORT_IS_OUTPUT,
				0
			);
			if port.is_null() {
				jack::jack_client_close( jack );
				return Err( io::Error::new( io::ErrorKind::Other, "" ) );
			}

			let mut this = Box::new( Player{
				shared: sync::Mutex::new( SharedData{
					events: Vec::new(),
					changed: false,
				} ),
				jack: jack,
				port: port,
				name: name.into(),
			} );

			if jack::jack_set_process_callback( this.jack, Player::process_callback, &mut *this ) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "" ) );
			}
			if jack::jack_set_sync_callback( this.jack, Player::sync_callback, &mut *this ) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "" ) );
			}

			if jack::jack_activate( this.jack ) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "" ) );
			}

			Ok( this )
		}
	}

	pub fn set_data( &mut self, events: Vec<midi::Event> ) {
		let mut shared = self.shared.lock().unwrap();
		shared.events = events;
		shared.changed = true;
	}

	pub fn connect( &self, port: &str ) -> io::Result<()> {
		unsafe {
			if jack::jack_connect(
				self.jack,
				ffi::CString::new( format!( "{}:out", self.name ) ).unwrap().as_ptr(),
				ffi::CString::new( port ).unwrap().as_ptr(),
			) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "" ) );
			}
		}
		Ok( () )
	}

	pub fn play( &mut self ) -> io::Result<()> {
		unsafe {
			jack::jack_transport_start( self.jack );
		}
		Ok( () )
	}

	pub fn seek( &mut self, time: ratio::Ratio ) -> io::Result<()> {
		unsafe {
			let mut pos: jack::Position = mem::uninitialized();
			jack::jack_transport_query( self.jack, &mut pos );
			jack::jack_transport_locate( self.jack, (time * pos.frame_rate as i64).to_int() as u32 );
		}
		Ok( () )
	}

	extern fn process_callback( size: u32, this: *mut any::Any ) -> i32 {
		unsafe {
			let this = &mut *(this as *mut Player);

			let mut pos: jack::Position = mem::uninitialized();
			match jack::jack_transport_query( this.jack, &mut pos ) {
				jack::TransportState::Rolling => (),
				_ => return 0,
			}

			let buf = jack::jack_port_get_buffer( this.port, size );
			jack::jack_midi_clear_buffer( buf );

			let mut shared = match this.shared.try_lock() {
				Err( _ ) => return 0,
				Ok ( v ) => v,
			};

			if shared.changed {
				for ch in 0 .. 16 {
					let msg: [u8; 3] = [ 0xb0 + ch, 0x7b, 0x00 ];
					jack::jack_midi_event_write( buf, 0, &msg as *const u8, msg.len() );
				}
				shared.changed = false;
			}

			let fbgn = ratio::Ratio::new( (pos.frame       ) as i64, pos.frame_rate as i64 );
			let fend = ratio::Ratio::new( (pos.frame + size) as i64, pos.frame_rate as i64 );
			let ibgn = misc::bsearch_boundary( &shared.events, |e| e.time < fbgn );
			let iend = misc::bsearch_boundary( &shared.events, |e| e.time < fend );
			for ev in shared.events[ibgn .. iend].iter() {
				let n = (ev.time * pos.frame_rate as i64).to_int() as u32 - pos.frame;
				jack::jack_midi_event_write( buf, n, &ev.msg as *const u8, ev.len as usize );
			}
		}
		0
	}

	extern fn sync_callback( _: jack::TransportState, _: *mut jack::Position, this_ptr: *mut any::Any ) -> i32 {
		unsafe {
			let this = &mut *(this_ptr as *mut Player);

			let mut shared = match this.shared.try_lock() {
				Err( _ ) => return 0,
				Ok ( v ) => v,
			};

			shared.changed = true;
			1 // ready to roll.
		}
	}
}
