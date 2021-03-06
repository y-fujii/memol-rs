// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;

#[cfg(target_os = "linux")]
pub fn wait_file<T: AsRef<path::Path>>(path: T) -> io::Result<()> {
    const IN_CLOEXEC: i32 = 0o2000000;
    const IN_CLOSE_WRITE: u32 = 0x8;
    extern "C" {
        fn inotify_init1(_: i32) -> i32;
        fn inotify_add_watch(_: i32, _: *const os::raw::c_char, _: u32) -> i32;
    }
    use std::io::prelude::*;
    use std::os::unix::io::FromRawFd;

    let path = match path.as_ref().to_str() {
        Some(e) => e,
        None => return Err(io::ErrorKind::Other.into()),
    };
    let path = ffi::CString::new(path).unwrap();

    unsafe {
        let fd = inotify_init1(IN_CLOEXEC);
        let mut fs = fs::File::from_raw_fd(fd);

        if inotify_add_watch(fd, path.as_ptr(), IN_CLOSE_WRITE) < 0 {
            return Err(io::ErrorKind::Other.into());
        }

        let _ = fs.read(&mut [0])?;
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn wait_file<T: AsRef<path::Path>>(path: T) -> io::Result<()> {
    let path = path.as_ref();
    let bgn = fs::metadata(path)?.modified()?;
    let mut mid;
    loop {
        thread::sleep(time::Duration::from_millis(100));
        mid = fs::metadata(path)?.modified()?;
        if mid != bgn {
            break;
        }
    }
    loop {
        thread::sleep(time::Duration::from_millis(100));
        let end = fs::metadata(path)?.modified()?;
        if end == mid {
            break;
        }
        mid = end;
    }
    Ok(())
}

pub enum WaitResult<T> {
    File(time::SystemTime),
    Channel(T),
    Disconnect,
}

pub fn wait_file_or_channel<T: AsRef<path::Path>, U>(
    path: &T,
    rx: &sync::mpsc::Receiver<U>,
    bgn: time::SystemTime,
) -> WaitResult<U> {
    let mut mid;
    loop {
        mid = fs::metadata(path).and_then(|e| e.modified()).unwrap_or(bgn);
        if mid != bgn {
            break;
        }
        match rx.recv_timeout(time::Duration::from_millis(100)) {
            Ok(v) => return WaitResult::Channel(v),
            Err(sync::mpsc::RecvTimeoutError::Timeout) => (),
            Err(sync::mpsc::RecvTimeoutError::Disconnected) => return WaitResult::Disconnect,
        }
    }
    loop {
        match rx.recv_timeout(time::Duration::from_millis(100)) {
            Ok(v) => return WaitResult::Channel(v),
            Err(sync::mpsc::RecvTimeoutError::Timeout) => (),
            Err(sync::mpsc::RecvTimeoutError::Disconnected) => return WaitResult::Disconnect,
        }
        let end = fs::metadata(path).and_then(|e| e.modified()).unwrap_or(mid);
        if end == mid {
            break;
        }
        mid = end;
    }
    WaitResult::File(mid)
}
