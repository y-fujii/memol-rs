// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::player;
use memol::*;
use std::*;
use windows::Win32::Media;
use windows::Win32::Media::Audio;

const TPS: u32 = 2880;

pub struct Player {
    offset: f64,
    events: Vec<midi::Event>,
    stream: Option<(String, Audio::HMIDISTRM)>,
    buffer: Vec<(Vec<u32>, Box<Audio::MIDIHDR>)>,
}

unsafe impl Send for Player {}

impl player::Player for Player {
    fn on_received_boxed(&mut self, _: Box<dyn 'static + Fn(&[midi::Event]) + Send>) {}

    fn set_data(&mut self, events: &[midi::Event]) {
        self.clear_buffer();
        let mut events = events.to_vec();
        let t_max = events.iter().map(|e| e.time).fold(0.0, f64::max);
        smf::time_code_add(&mut events, 0.0, t_max);
        events.sort_by(|x, y| (x.time, x.prio).partial_cmp(&(y.time, y.prio)).unwrap());
        self.events = events;
    }

    fn ports_from(&mut self) -> io::Result<Vec<(String, bool)>> {
        Err(io::ErrorKind::Other.into())
    }

    fn connect_from(&mut self, _: &str) -> io::Result<()> {
        Err(io::ErrorKind::Other.into())
    }

    fn disconnect_from(&mut self, _: &str) -> io::Result<()> {
        Err(io::ErrorKind::Other.into())
    }

    fn ports_to(&mut self) -> io::Result<Vec<(String, bool)>> {
        let mut dst = Vec::new();
        let n = unsafe { Audio::midiOutGetNumDevs() };
        for i in 0..n {
            let Some(name) = Self::out_device_name(i) else {
                continue;
            };
            let is_conn = self.is_connected_to(&name);
            dst.push((name, is_conn));
        }
        Ok(dst)
    }

    fn connect_to(&mut self, a_name: &str) -> io::Result<()> {
        unsafe {
            let n = Audio::midiOutGetNumDevs();
            for i in 0..n {
                let Some(name) = Self::out_device_name(i) else {
                    continue;
                };
                if name != a_name {
                    continue;
                }
                let mut stream = Audio::HMIDISTRM::default();
                if Audio::midiStreamOpen(&mut stream, &mut [i], None, None, Audio::CALLBACK_NULL.0) != 0 {
                    continue;
                }
                if Audio::midiStreamProperty(
                    stream,
                    &mut Audio::MIDIPROPTIMEDIV {
                        cbStruct: mem::size_of::<Audio::MIDIPROPTIMEDIV>() as u32,
                        dwTimeDiv: TPS / 2,
                    } as *mut _ as *mut u8,
                    (Audio::MIDIPROP_SET | Audio::MIDIPROP_TIMEDIV) as u32,
                ) != 0
                {
                    Audio::midiStreamClose(stream);
                    continue;
                }
                self.close();
                self.stream = Some((name, stream));
                return Ok(());
            }
        }
        Err(io::ErrorKind::Other.into())
    }

    fn disconnect_to(&mut self, name: &str) -> io::Result<()> {
        if self.is_connected_to(name) {
            self.close();
        }
        Ok(())
    }

    fn send(&mut self, events: &[midi::Event]) {
        let Some((_, stream)) = self.stream else {
            return;
        };
        for ev in events {
            let msg = u32::from_le_bytes([ev.msg[0], ev.msg[1], ev.msg[2], 0]);
            unsafe { Audio::midiOutShortMsg(Audio::HMIDIOUT(stream.0), msg) };
        }
    }

    fn play(&mut self) -> io::Result<()> {
        let Some((_, stream)) = self.stream else {
            return Err(io::Error::other("Output is not connected."));
        };
        self.clear_buffer();

        let mut t0 = (TPS as f64 * self.offset).round() as i64;
        let mut index = 0;
        while index < self.events.len() {
            let mut chunk = Vec::new();
            while index < self.events.len() && chunk.len() <= 16384 - 3 {
                let ev = &self.events[index];
                index += 1;
                let t1 = (TPS as f64 * ev.time).round() as i64;
                if t1 < t0 {
                    continue;
                }
                chunk.push((t1 - t0) as u32);
                chunk.push(0);
                chunk.push(u32::from_le_bytes([ev.msg[0], ev.msg[1], ev.msg[2], 0]));
                t0 = t1;
            }

            unsafe {
                let n_bytes = 4 * chunk.len();
                let mut header = Box::new(Audio::MIDIHDR {
                    lpData: mem::transmute(chunk.as_ptr()),
                    dwBufferLength: n_bytes as u32,
                    dwBytesRecorded: n_bytes as u32,
                    dwFlags: 0,
                    ..Audio::MIDIHDR::default()
                });
                if Audio::midiOutPrepareHeader(
                    Audio::HMIDIOUT(stream.0),
                    &mut *header,
                    mem::size_of::<Audio::MIDIHDR>() as u32,
                ) != 0
                {
                    return Err(io::Error::other("midiOutPrepareHeader()."));
                }
                if Audio::midiStreamOut(stream, &mut *header, mem::size_of::<Audio::MIDIHDR>() as u32) != 0 {
                    Audio::midiOutUnprepareHeader(
                        Audio::HMIDIOUT(stream.0),
                        &mut *header,
                        mem::size_of::<Audio::MIDIHDR>() as u32,
                    );
                    return Err(io::Error::other("midiStreamOut()."));
                }
                self.buffer.push((chunk, header));
            }
        }
        if unsafe { Audio::midiStreamRestart(stream) } != 0 {
            return Err(io::Error::other("midiStreamRestart()."));
        }

        Ok(())
    }

    fn stop(&mut self) {
        self.clear_buffer();
    }

    fn seek(&mut self, offset: f64) {
        self.clear_buffer();
        self.offset = offset;
    }

    fn status(&mut self) -> (bool, f64) {
        if self.buffer.is_empty() {
            (false, self.offset)
        } else {
            let (_, stream) = self.stream.as_ref().unwrap();
            let dt = Self::position(*stream);
            (true, self.offset + dt)
        }
    }

    fn info(&mut self) -> String {
        String::new()
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.close();
    }
}

impl Player {
    pub fn new() -> Result<Self, Box<dyn error::Error>> {
        Ok(Player {
            offset: 0.0,
            events: Vec::new(),
            stream: None,
            buffer: Vec::new(),
        })
    }

    fn clear_buffer(&mut self) {
        let Some((_, stream)) = self.stream else {
            return;
        };
        for (buffer, mut header) in self.buffer.drain(..) {
            unsafe {
                Audio::midiStreamPause(stream);
                self.offset += Self::position(stream);
                Audio::midiStreamStop(stream);
                Audio::midiOutUnprepareHeader(
                    Audio::HMIDIOUT(stream.0),
                    &mut *header,
                    mem::size_of::<Audio::MIDIHDR>() as u32,
                );
            }
            mem::drop(buffer);
        }
    }

    fn close(&mut self) {
        self.clear_buffer();
        if let Some((_, stream)) = self.stream.take() {
            unsafe { Audio::midiStreamClose(stream) };
        }
    }

    fn is_connected_to(&self, a_name: &str) -> bool {
        match self.stream {
            Some((ref name, _)) => name == a_name,
            _ => false,
        }
    }

    fn out_device_name(n: u32) -> Option<String> {
        let mut caps = Audio::MIDIOUTCAPSW::default();
        if unsafe { Audio::midiOutGetDevCapsW(n as usize, &mut caps, mem::size_of::<Audio::MIDIOUTCAPSW>() as u32) }
            != 0
        {
            return None;
        }
        // caps.szPname is not aligned.
        let name: Vec<u16> = caps.szPname.into_iter().take_while(|c| *c != 0).collect();
        Some(String::from_utf16_lossy(&name))
    }

    fn position(stream: Audio::HMIDISTRM) -> f64 {
        let mut mmtime = Media::MMTIME {
            wType: Media::TIME_TICKS,
            ..Default::default()
        };
        unsafe {
            Audio::midiStreamPosition(stream, &mut mmtime, mem::size_of::<Media::MMTIME>() as u32);
            mmtime.u.ticks as f64 / TPS as f64
        }
    }
}
