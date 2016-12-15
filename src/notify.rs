// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD license.
use std::*;
use std::io::prelude::*;
use std::os::unix::io::FromRawFd;
use cext;


pub fn notify_wait( path: &str ) -> io::Result<()> {
	unsafe {
		let fd = cext::inotify_init1( cext::IN_CLOEXEC as os::raw::c_int );
		let mut fs = fs::File::from_raw_fd( fd );

		if cext::inotify_add_watch(
			fd,
			ffi::CString::new( path ).unwrap().as_ptr(),
			cext::IN_CLOSE_WRITE
		) < 0 {
			return Err( io::Error::new( io::ErrorKind::Other, "" ) );
		}

		let mut buf: [u8; 4096] = mem::uninitialized();
		let _ = fs.read( &mut buf )?; // XXX
	}
	Ok( () )
}
