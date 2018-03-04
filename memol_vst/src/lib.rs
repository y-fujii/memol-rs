// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
#[macro_use]
extern crate vst;
extern crate memol;
extern crate memol_cli;
mod events;
use std::*;
use vst::host::Host;
use memol::{ misc, midi };


const REMOTE_ADDR: &'static str = "ws://127.0.0.1:27182";

struct SharedData {
	events: Vec<midi::Event>,
	changed: bool,
}

struct Plugin {
	host: vst::plugin::HostCallback,
	shared: sync::Arc<sync::Mutex<SharedData>>,
	buffer: events::EventBuffer,
	playing: bool,
}

impl default::Default for Plugin {
	fn default() -> Self {
		Plugin{
			host: vst::plugin::HostCallback::default(),
			shared: sync::Arc::new( sync::Mutex::new( SharedData{
				events: Vec::new(),
				changed: false,
			} ) ),
			buffer: events::EventBuffer::new(),
			playing: false,
		}
	}
}

impl vst::plugin::Plugin for Plugin {
	fn new( host: vst::plugin::HostCallback ) -> Self {
		let shared = sync::Arc::new( sync::Mutex::new( SharedData{
			events: Vec::new(),
			changed: false,
		} ) );

		{
			let shared = shared.clone();
			// XXX: finalization.
			thread::spawn( move || {
				loop {
					memol_cli::ipc::Bus::new().connect( REMOTE_ADDR, |msg| {
						if let memol_cli::ipc::Message::Success{ events: evs } = msg {
							let mut shared = shared.lock().unwrap();
							shared.events = evs.into_iter().map( |e| e.into() ).collect();
							shared.changed = true;
						}
					} ).ok();
					thread::sleep( time::Duration::new( 3, 0 ) );
				}
			} );
		}

		Plugin{
			host: host,
			shared: shared,
			buffer: events::EventBuffer::new(),
			playing: false,
		}
	}

	fn get_info( &self ) -> vst::plugin::Info {
		vst::plugin::Info{
			name: "memol".into(),
			unique_id: 271828182,
			inputs:  0,
			outputs: 0,
			category: vst::plugin::Category::Synth,
			.. Default::default()
		}
	}

	fn can_do( &self, can_do: vst::plugin::CanDo ) -> vst::api::Supported {
		match can_do {
			vst::plugin::CanDo::SendEvents |
			vst::plugin::CanDo::SendMidiEvent |
			vst::plugin::CanDo::ReceiveTimeInfo =>
				vst::api::Supported::Yes,
			_ =>
				vst::api::Supported::No,
		}
	}

	fn process( &mut self, _: &mut vst::buffer::AudioBuffer<f32> ) {
		self.buffer.clear();

		let size = self.host.get_block_size() as f64;
		let info = match self.host.get_time_info( 0 ) {
			Some( v ) => v,
			None      => return,
		};
		let playing = info.flags & vst::api::flags::TRANSPORT_PLAYING.bits() != 0;

		let mut shared = match self.shared.try_lock() {
			Ok ( v ) => v,
			Err( _ ) => return,
		};

		if shared.changed || playing != self.playing {
			for ch in 0 .. 16 {
				// all notes off message.
				self.buffer.push( &[ 0xb0 + ch, 0x7b, 0x00 ], 0 );
			}
			shared.changed = false;
		}

		if playing {
			let frame = |ev: &midi::Event| (ev.time * info.sample_rate - info.sample_pos).round();
			let ibgn = misc::bsearch_boundary( &shared.events, |ev| (frame( ev ), ev.prio) < (0.0,  i16::MIN) );
			let iend = misc::bsearch_boundary( &shared.events, |ev| (frame( ev ), ev.prio) < (size, i16::MIN) );
			for ev in shared.events[ibgn .. iend].iter() {
				self.buffer.push( &ev.msg, frame( ev ) as i32 );
			}
		}

		self.host.process_events( self.buffer.events() );
		self.playing = playing;
	}
}

plugin_main!( Plugin );
