// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD license.
use std::*;
use cext;
use misc;
use midi;


// XXX: unmovable mark or 2nd depth indirection.
pub struct Player {
	data: Vec<midi::Event>,
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

			let mut this = Box::new( Player{
				data: Vec::new(),
				jack: jack,
				port: port,
			} );

			if cext::jack_set_process_callback(
				jack,
				Some( Player::callback ),
				&mut *this as *mut Player as *mut os::raw::c_void
			) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "" ) );
			}

			Ok( this )
		}
	}

	// XXX
	pub fn activate( &mut self ) -> io::Result<()> {
		unsafe {
			if cext::jack_activate( self.jack ) != 0 {
				return Err( io::Error::new( io::ErrorKind::Other, "" ) );
			}
		}
		Ok( () )
	}

	// XXX
	pub fn set_data( &mut self, data: Vec<midi::Event> ) {
		self.data = data;
	}

	extern fn callback( n: cext::jack_nframes_t, this_ptr: *mut os::raw::c_void ) -> os::raw::c_int {
		unsafe {
			let this = &*(this_ptr as *mut Player);

			let mut pos: cext::jack_position_t = mem::uninitialized();
			if cext::jack_transport_query( this.jack, &mut pos ) != cext::JackTransportRolling {
				return 0;
			}

			let buf = cext::jack_port_get_buffer( this.port, n );
			cext::jack_midi_clear_buffer( buf );

			/*
			let bgn = misc::lower_bound( &this.data, &(pos.frame + 0), |x, y| (x.time as u32) < *y );
			let end = misc::lower_bound( &this.data, &(pos.frame + n), |x, y| (x.time as u32) < *y );
			for i in bgn .. end {
				cext::jack_midi_event_write(
					buf,
					this.data[i].time as u32 - pos.frame,
					&this.data[i].msg as *const u8,
					this.data[i].len,
				);
			}
			*/
			for ev in this.data.iter() {
				let t = ev.time as u32;
				if pos.frame <= t && t < pos.frame + n {
					cext::jack_midi_event_write( buf, t - pos.frame, &ev.msg as *const u8, ev.len );
				}
			}
		}
		0
	}
}
