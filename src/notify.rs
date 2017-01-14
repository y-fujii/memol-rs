// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::*;


#[cfg( target_os = "linux" )]
pub fn notify_wait( path: &str ) -> io::Result<()> {
	const IN_CLOEXEC: i32 = 0o2000000;
	const IN_CLOSE_WRITE: u32 = 0x8;
	extern "C" {
		fn inotify_init1( _: i32 ) -> i32;
		fn inotify_add_watch( _: i32, _: *const os::raw::c_char, _: u32 ) -> i32;
	}
	use std::io::prelude::*;
	use std::os::unix::io::FromRawFd;

	unsafe {
		let fd = inotify_init1( IN_CLOEXEC );
		let mut fs = fs::File::from_raw_fd( fd );

		if inotify_add_watch( fd, ffi::CString::new( path ).unwrap().as_ptr(), IN_CLOSE_WRITE ) < 0 {
			return Err( io::Error::new( io::ErrorKind::Other, "" ) );
		}

		let mut buf: [u8; 4096] = mem::uninitialized();
		let _ = fs.read( &mut buf )?; // XXX
	}
	Ok( () )
}

#[cfg( not( target_os = "linux" ) )]
pub fn notify_wait( path: &str ) -> io::Result<()> {
	let bgn = fs::metadata( path )?.modified()?;
	let mut mid;
	loop {
		thread::sleep( time::Duration::from_millis( 100 ) );
		mid = fs::metadata( path )?.modified()?;
		if mid != bgn {
			break;
		}
	}
	loop {
		thread::sleep( time::Duration::from_millis( 100 ) );
		let end = fs::metadata( path )?.modified()?;
		if end == mid {
			break;
		}
		mid = end;
	}
	Ok( () )
}
