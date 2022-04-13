// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;

const BUFFER_SIZE: usize = 4096;

// derived from vst::api::Events.
#[repr(C)]
struct Events {
    num_events: i32,
    _reserved: isize,
    events: [*mut vst::api::MidiEvent; BUFFER_SIZE],
}

pub struct EventBuffer {
    holder: Box<[vst::api::MidiEvent; BUFFER_SIZE]>,
    buffer: Box<Events>,
}

unsafe impl Send for Events {}

impl EventBuffer {
    pub fn new() -> Self {
        unsafe {
            let mut this = EventBuffer {
                holder: Box::new(mem::zeroed()),
                buffer: Box::new(Events {
                    num_events: 0,
                    _reserved: 0,
                    events: [ptr::null_mut(); BUFFER_SIZE],
                }),
            };
            for i in 0..BUFFER_SIZE {
                this.holder[i].event_type = vst::api::EventType::Midi;
                this.holder[i].byte_size = mem::size_of::<vst::api::MidiEvent>() as i32;
                this.buffer.events[i] = &mut this.holder[i];
            }
            this
        }
    }

    pub fn clear(&mut self) {
        self.buffer.num_events = 0;
    }

    pub fn push(&mut self, msg: &[u8], frame: i32) {
        let ev = &mut self.holder[self.buffer.num_events as usize];
        ev.delta_frames = frame;
        ev.midi_data.copy_from_slice(&msg[..3]);
        self.buffer.num_events += 1;
    }

    pub fn events(&self) -> &vst::api::Events {
        unsafe { mem::transmute(&*self.buffer) }
    }
}
