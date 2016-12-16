// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD license.
use std::*;
use cext;
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
	jack: *mut cext::jack_client_t,
	port: *mut cext::jack_port_t,
	name: String,
}

impl Drop for Player {
	fn drop( &mut self ) {
		unsafe {
			cext::jack_client_close( self.jack );
		}
	}
}

impl Player {
	pub fn new( name: &str ) -> io::Result<Box<Player>> {
		unsafe {
			let jack = cext::jack_client_open(
				ffi::CString::new( name ).unwrap().as_ptr(),
				cext::JackOptions::JackNullOption,
				ptr::null_mut()
			);
			if jack.is_null() {
				return Err( io::Error::new( io::ErrorKind::Other, "" ) );
			}

			let port = cext::jack_port_register(
				jack,
				ffi::CString::new( "out" ).unwrap().as_ptr(),
				cext::JACK_DEFAULT_MIDI_TYPE.as_ptr() as *const i8,
				cext::JackPortFlags::JackPortIsOutput as u64,
				0
			);
			if port.is_null() {
				cext::jack_client_close( jack );
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

			let this_ptr = &mut *this as *mut Player as *mut os::raw::c_void;
			if cext::jack_set_process_callback( this.jack, Some( Player::process_callback ), this_ptr ) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "" ) );
			}
			if cext::jack_set_sync_callback( this.jack, Some( Player::sync_callback ), this_ptr ) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "" ) );
			}

			if cext::jack_activate( this.jack ) != 0 {
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
			if cext::jack_connect(
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
			cext::jack_transport_start( self.jack );
		}
		Ok( () )
	}

	pub fn seek( &mut self, time: ratio::Ratio ) -> io::Result<()> {
		unsafe {
			let mut pos: cext::jack_position_t = mem::uninitialized();
			cext::jack_transport_query( self.jack, &mut pos );
			cext::jack_transport_locate( self.jack, (time * pos.frame_rate as i64).to_int() as u32 );
		}
		Ok( () )
	}

	extern fn process_callback( size: cext::jack_nframes_t, this_ptr: *mut os::raw::c_void ) -> os::raw::c_int {
		unsafe {
			let this = &mut *(this_ptr as *mut Player);

			let mut pos: cext::jack_position_t = mem::uninitialized();
			if cext::jack_transport_query( this.jack, &mut pos ) != cext::JackTransportRolling {
				return 0;
			}

			let buf = cext::jack_port_get_buffer( this.port, size );
			cext::jack_midi_clear_buffer( buf );

			let mut shared = match this.shared.try_lock() {
				Err( _ ) => return 0,
				Ok ( v ) => v,
			};

			if shared.changed {
				for ch in 0 .. 16 {
					let msg: [u8; 3] = [ 0xb0 + ch, 0x7b, 0x00 ];
					cext::jack_midi_event_write( buf, 0, &msg as *const u8, msg.len() );
				}
				shared.changed = false;
			}

			let fbgn = ratio::Ratio::new( (pos.frame       ) as i64, pos.frame_rate as i64 );
			let fend = ratio::Ratio::new( (pos.frame + size) as i64, pos.frame_rate as i64 );
			let ibgn = misc::bsearch_boundary( &shared.events, |e| e.time < fbgn );
			let iend = misc::bsearch_boundary( &shared.events, |e| e.time < fend );
			for ev in shared.events[ibgn .. iend].iter() {
				let n = (ev.time * pos.frame_rate as i64).to_int() as u32 - pos.frame;
				cext::jack_midi_event_write( buf, n, &ev.msg as *const u8, ev.len as usize );
			}
		}
		0
	}

	extern fn sync_callback( _: cext::jack_transport_state_t, _: *mut cext::jack_position_t, this_ptr: *mut os::raw::c_void ) -> i32 {
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
