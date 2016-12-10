use std::*;
use cext;


// XXX: unmovable mark or 2nd depth indirection.
pub struct Player {
	jack: *mut cext::jack_client_t,
	port: *mut cext::jack_port_t,
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

			let mut this = Box::new( Player{ jack: jack, port: port } );

			if cext::jack_set_process_callback(
				jack,
				Some( Player::callback ),
				&mut *this as *mut Player as *mut os::raw::c_void
			) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "" ) );
			}

			if cext::jack_activate( jack ) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "" ) );
			}

			Ok( this )
		}
	}

	extern fn callback( _: cext::jack_nframes_t, this_ptr: *mut os::raw::c_void ) -> os::raw::c_int {
		let _this = unsafe { &*(this_ptr as *mut Player) };
		let _ = _this.port;
		0
	}
}
