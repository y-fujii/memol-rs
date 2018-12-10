// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use vst::plugin_main;
use vst::host::Host;
use memol::{ misc, midi };
mod events;


const REMOTE_ADDR: &'static str = "ws://127.0.0.1:27182";

struct SharedData {
	events: Vec<midi::Event>,
	immediate: Vec<midi::Event>,
	changed: bool,
}

struct Plugin {
	host: vst::plugin::HostCallback,
	buffer: events::EventBuffer,
	shared: sync::Arc<sync::Mutex<SharedData>>,
	playing: bool,
	location: isize,
}

impl default::Default for Plugin {
	fn default() -> Self {
		Plugin{
			host: vst::plugin::HostCallback::default(),
			buffer: events::EventBuffer::new(),
			shared: sync::Arc::new( sync::Mutex::new( SharedData{
				events: Vec::new(),
				immediate: Vec::new(),
				changed: false,
			} ) ),
			playing: false,
			location: 0,
		}
	}
}

impl vst::plugin::Plugin for Plugin {
	fn new( host: vst::plugin::HostCallback ) -> Self {
		let shared = sync::Arc::new( sync::Mutex::new( SharedData{
			events: Vec::new(),
			immediate: Vec::new(),
			changed: false,
		} ) );

		{
			let shared = shared.clone();
			// XXX: finalization.
			thread::spawn( move || {
				loop {
					memol_cli::ipc::Bus::new().connect( REMOTE_ADDR, |msg| {
						match msg {
							memol_cli::ipc::Message::Success{ events: evs } => {
								let mut shared = shared.lock().unwrap();
								shared.events = evs.into_iter().map( |e| e.into() ).collect();
								shared.changed = true;
							},
							memol_cli::ipc::Message::Failure{ .. } => {
								let mut shared = shared.lock().unwrap();
								shared.events.clear();
								shared.changed = true;
							},
							memol_cli::ipc::Message::Immediate{ events: evs } => {
								let mut shared = shared.lock().unwrap();
								shared.immediate.extend( evs.into_iter().map( |e| e.into() ) );
							},
							_ => (),
						}
					} ).ok();
					thread::sleep( time::Duration::new( 3, 0 ) );
				}
			} );
		}

		Plugin{
			host: host,
			buffer: events::EventBuffer::new(),
			shared: shared,
			playing: false,
			location: 0,
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

	fn process( &mut self, buffer: &mut vst::buffer::AudioBuffer<'_, f32> ) {
		self.buffer.clear();

		let size = buffer.samples() as isize;
		let info = match self.host.get_time_info( 0 ) {
			Some( v ) => v,
			None      => return,
		};
		let playing  = info.flags & vst::api::flags::TRANSPORT_PLAYING.bits() != 0;
		let location = info.sample_pos.round() as isize;

		let mut shared = match self.shared.try_lock() {
			Ok ( v ) => v,
			Err( _ ) => return,
		};

		if shared.changed || playing != self.playing || (playing && location != self.location) {
			for ch in 0 .. 16 {
				// all sound off.
				self.buffer.push( &[ 0xb0 + ch, 0x78, 0x00 ], 0 );
				// reset all controllers.
				self.buffer.push( &[ 0xb0 + ch, 0x79, 0x00 ], 0 );
			}
			shared.changed = false;
		}

		for ev in shared.immediate.drain( .. ) {
			self.buffer.push( &ev.msg, 0 );
		}

		if playing {
			let frame = |ev: &midi::Event| (ev.time * info.sample_rate).round() as isize - location;
			let ibgn = misc::bsearch_boundary( &shared.events, |ev| frame( ev ) < 0    );
			let iend = misc::bsearch_boundary( &shared.events, |ev| frame( ev ) < size );
			for ev in shared.events[ibgn .. iend].iter() {
				self.buffer.push( &ev.msg, frame( ev ) as i32 );
			}
		}

		self.host.process_events( self.buffer.events() );
		self.playing  = playing;
		self.location = location + size;
	}
}

plugin_main!( Plugin );
