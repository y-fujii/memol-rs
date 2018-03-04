// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use vst;


const BUFFER_SIZE: usize = 4096;

// derived from vst::api::Events.
#[repr( C )]
struct Events {
	num_events: i32,
	_reserved: isize,
	events: [*mut vst::api::MidiEvent; BUFFER_SIZE],
}

pub struct EventBuffer {
	holder: Box<[vst::api::MidiEvent; BUFFER_SIZE]>,
	buffer: Box<Events>,
}

impl EventBuffer {
	pub fn new() -> Self {
		let mut this = unsafe {
			EventBuffer{
				holder: Box::new( mem::uninitialized() ),
				buffer: Box::new( Events{
					num_events: 0,
					_reserved: 0,
					events: mem::uninitialized(),
				} ),
			}
		};
		for i in 0 .. BUFFER_SIZE {
			(*this.holder)[i] = vst::api::MidiEvent{
				event_type: vst::api::EventType::Midi,
				byte_size: mem::size_of::<vst::api::MidiEvent>() as i32,
				delta_frames: 0,
				flags: 0,
				note_length: 0,
				note_offset: 0,
				midi_data: [ 0, 0, 0 ],
				_midi_reserved: 0,
				detune: 0,
				note_off_velocity: 0,
				_reserved1: 0,
				_reserved2: 0,
			};
			this.buffer.events[i] = &mut (*this.holder)[i];
		}
		this
	}

	pub fn clear( &mut self ) {
		self.buffer.num_events = 0;
	}

	pub fn push( &mut self, msg: &[u8], frame: i32 ) {
		let ev = &mut self.holder[self.buffer.num_events as usize];
		ev.delta_frames = frame;
		for i in 0 .. 3 {
			ev.midi_data[i] = msg[i];
		}
		self.buffer.num_events += 1;
	}

	pub fn events( &self ) -> &vst::api::Events {
		unsafe {
			mem::transmute( &*self.buffer )
		}
	}
}