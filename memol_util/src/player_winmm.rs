// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::player;
use memol::*;
use std::*;
use windows::Win32::Media;
use windows::Win32::Media::Audio;

const TPS: u32 = 2880;

struct Internal {
    name: String,
    stream: Audio::HMIDISTRM,
    buffer: Vec<(Vec<u32>, Box<Audio::MIDIHDR>)>,
}

pub struct Player {
    is_playing: bool,
    offset: f64,
    events: Vec<midi::Event>,
    internal: Option<Internal>,
}

impl Drop for Internal {
    fn drop(&mut self) {
        self.clear();
        unsafe { Audio::midiStreamClose(self.stream) };
    }
}

impl Internal {
    fn out_devices() -> impl Iterator<Item = (usize, String)> {
        unsafe {
            let n = Audio::midiOutGetNumDevs() as usize;
            (0..n).filter_map(|i| {
                let mut caps = Audio::MIDIOUTCAPSW::default();
                if Audio::midiOutGetDevCapsW(i, &mut caps, mem::size_of::<Audio::MIDIOUTCAPSW>() as u32) != 0 {
                    return None;
                }
                // caps.szPname is not aligned.
                let name: Vec<u16> = caps.szPname.into_iter().take_while(|c| *c != 0).collect();
                Some((i, String::from_utf16_lossy(&name)))
            })
        }
    }

    fn new(name: &str, time_div: u32) -> io::Result<Internal> {
        let Some(dev) = Self::out_devices().find(|(_, n)| n == name) else {
            return Err(io::ErrorKind::Other.into());
        };

        let mut stream = Audio::HMIDISTRM::default();
        unsafe {
            if Audio::midiStreamOpen(&mut stream, &mut [dev.0 as u32], None, None, Audio::CALLBACK_NULL.0) != 0 {
                return Err(io::Error::other("midiStreamOpen()."));
            }
            if Audio::midiStreamProperty(
                stream,
                &mut Audio::MIDIPROPTIMEDIV {
                    cbStruct: mem::size_of::<Audio::MIDIPROPTIMEDIV>() as u32,
                    dwTimeDiv: time_div,
                } as *mut _ as *mut u8,
                (Audio::MIDIPROP_SET | Audio::MIDIPROP_TIMEDIV) as u32,
            ) != 0
            {
                Audio::midiStreamClose(stream);
                return Err(io::Error::other("midiStreamProperty()."));
            }
        }

        Ok(Internal {
            name: dev.1,
            stream: stream,
            buffer: Vec::new(),
        })
    }

    fn clear(&mut self) {
        unsafe {
            Audio::midiStreamStop(self.stream);
            for (buffer, mut header) in self.buffer.drain(..) {
                assert!(header.dwFlags & Audio::MHDR_DONE != 0);
                Audio::midiOutUnprepareHeader(
                    Audio::HMIDIOUT(self.stream.0),
                    &mut *header,
                    mem::size_of::<Audio::MIDIHDR>() as u32,
                );
                mem::drop(buffer);
            }
        }
    }

    fn add_chunk(&mut self, chunk: Vec<u32>) -> io::Result<()> {
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
                Audio::HMIDIOUT(self.stream.0),
                &mut *header,
                mem::size_of::<Audio::MIDIHDR>() as u32,
            ) != 0
            {
                return Err(io::Error::other("midiOutPrepareHeader()."));
            }
            if Audio::midiStreamOut(self.stream, &mut *header, mem::size_of::<Audio::MIDIHDR>() as u32) != 0 {
                Audio::midiOutUnprepareHeader(
                    Audio::HMIDIOUT(self.stream.0),
                    &mut *header,
                    mem::size_of::<Audio::MIDIHDR>() as u32,
                );
                return Err(io::Error::other("midiStreamOut()."));
            }
            self.buffer.push((chunk, header));
        }
        Ok(())
    }

    fn add_data(&mut self, events: &[midi::Event], offset: f64) -> io::Result<()> {
        let mut t0 = (TPS as f64 * offset).round() as i64;
        let mut index = 0;
        while index < events.len() {
            let mut chunk = Vec::new();
            while index < events.len() && chunk.len() <= 16384 - 3 {
                let ev = &events[index];
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
            self.add_chunk(chunk)?;
        }
        Ok(())
    }

    fn add_time_code(&mut self, offset: f64) -> io::Result<()> {
        let b = smf::time_code_full(offset);
        let chunk = vec![
            0,
            0,
            (Audio::MEVT_LONGMSG as u32) << 24 | b.len() as u32,
            u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            u32::from_le_bytes([b[4], b[5], b[6], b[7]]),
            u32::from_le_bytes([b[8], b[9], 0, 0]),
        ];
        self.add_chunk(chunk)
    }

    fn pause(&mut self) {
        unsafe { Audio::midiStreamPause(self.stream) };
    }

    fn play(&self) -> io::Result<()> {
        if unsafe { Audio::midiStreamRestart(self.stream) } != 0 {
            return Err(io::Error::other("midiStreamRestart()."));
        }
        Ok(())
    }

    fn position(&self) -> f64 {
        let mut mmtime = Media::MMTIME {
            wType: Media::TIME_TICKS,
            ..Default::default()
        };
        unsafe {
            Audio::midiStreamPosition(self.stream, &mut mmtime, mem::size_of::<Media::MMTIME>() as u32);
            mmtime.u.ticks as f64 / TPS as f64
        }
    }
}

unsafe impl Send for Player {}

impl player::Player for Player {
    fn on_received_boxed(&mut self, _: Box<dyn 'static + Fn(&[midi::Event]) + Send>) {}

    fn set_data(&mut self, events: &[midi::Event]) {
        self.stop();

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
        for (_, name) in Internal::out_devices() {
            let is_conn = self.is_connected_to(&name);
            dst.push((name, is_conn));
        }
        Ok(dst)
    }

    fn connect_to(&mut self, name: &str) -> io::Result<()> {
        self.stop();
        self.internal = None;
        self.internal = Some(Internal::new(name, TPS / 2)?);
        self.stop();
        Ok(())
    }

    fn disconnect_to(&mut self, name: &str) -> io::Result<()> {
        if !self.is_connected_to(name) {
            return Err(io::ErrorKind::Other.into());
        }
        self.stop();
        self.internal = None;
        Ok(())
    }

    fn send(&mut self, events: &[midi::Event]) -> io::Result<()> {
        if self.is_playing {
            return Ok(());
        }
        let Some(ref mut internal) = self.internal else {
            return Err(io::ErrorKind::Other.into());
        };
        internal.clear();
        internal.add_data(events, 0.0)?;
        internal.play()
    }

    fn play(&mut self) -> io::Result<()> {
        if self.is_playing {
            return Ok(());
        }
        let Some(ref mut internal) = self.internal else {
            return Err(io::ErrorKind::Other.into());
        };
        internal.clear();
        internal.add_time_code(self.offset)?;
        internal.add_data(&self.events, self.offset)?;
        internal.play()?;
        self.is_playing = true;
        Ok(())
    }

    fn stop(&mut self) {
        if let Some(ref mut internal) = self.internal {
            internal.pause();
            if self.is_playing {
                self.offset += internal.position();
            }
            internal.clear();
            if internal.add_time_code(self.offset).is_ok() {
                internal.play().ok();
            }
        }
        self.is_playing = false;
    }

    fn seek(&mut self, offset: f64) {
        if let Some(ref mut internal) = self.internal {
            internal.clear();
            if internal.add_time_code(offset).is_ok() {
                internal.play().ok();
            }
        };
        self.offset = offset;
        self.is_playing = false;
    }

    fn status(&mut self) -> (bool, f64) {
        let Some(ref mut internal) = self.internal else {
            return (false, self.offset);
        };
        if self.is_playing {
            let pos = internal.position();
            (true, self.offset + pos)
        } else {
            (false, self.offset)
        }
    }

    fn info(&mut self) -> String {
        String::new()
    }
}

impl Player {
    pub fn new() -> Result<Self, Box<dyn error::Error>> {
        Ok(Player {
            is_playing: false,
            offset: 0.0,
            events: Vec::new(),
            internal: None,
        })
    }

    fn is_connected_to(&self, name: &str) -> bool {
        match self.internal {
            Some(ref internal) => internal.name == name,
            _ => false,
        }
    }
}
