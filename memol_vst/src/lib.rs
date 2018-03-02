// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
#[macro_use]
extern crate vst2;
extern crate memol;
extern crate memol_cli;
use std::*;
use memol::{ misc, midi };
use vst2::host::Host;


const REMOTE_ADDR: &'static str = "ws://localhost:27182";

struct HostCallbackExt {
	callback: Option<vst2::api::HostCallbackProc>,
	effect: *mut vst2::api::AEffect,
}

impl HostCallbackExt {
	fn callback( this: &vst2::plugin::HostCallback, op: vst2::host::OpCode, idx: i32, val: isize, ptr: *mut os::raw::c_void, opt: f32 ) -> isize {
		let this: &HostCallbackExt = unsafe { mem::transmute( this ) };
		this.callback.unwrap()( this.effect, op.into(), idx, val, ptr, opt )
	}

	fn get_time( this: &vst2::plugin::HostCallback ) -> Option<(f64, f64, bool)> {
		type Result = (f64, f64, [u8; 68], u32);

		let ptr = Self::callback( this, vst2::host::OpCode::GetTime, 0, 0, ptr::null_mut(), 0.0 );
		unsafe { (ptr as *const Result).as_ref() }.map( |&(pos, rate, _, flags)|
			(pos, rate, flags & 0b10 != 0)
		)
	}

	fn get_block_size( this: &vst2::plugin::HostCallback ) -> isize {
		Self::callback( this, vst2::host::OpCode::GetBlockSize, 0, 0, ptr::null_mut(), 0.0 )
	}
}

#[derive( Default )]
struct SharedData {
	events: Vec<midi::Event>,
	changed: bool,
}

#[derive( Default )]
struct Plugin {
	host: vst2::plugin::HostCallback,
	shared: sync::Arc<sync::Mutex<SharedData>>,
	is_playing: bool,
}

impl vst2::plugin::Plugin for Plugin {
	fn new( host: vst2::plugin::HostCallback ) -> Self {
		let shared = sync::Arc::new( sync::Mutex::new( SharedData{
			events: Vec::new(),
			changed: false,
		} ) );

		{
			let shared = shared.clone();
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
			is_playing: false,
		}
	}

	fn get_info( &self ) -> vst2::plugin::Info {
		vst2::plugin::Info{
			name: "memol".into(),
			unique_id: 271828182,
			inputs:  0,
			outputs: 0,
			category: vst2::plugin::Category::Synth,
			.. Default::default()
		}
	}

	fn can_do( &self, can_do: vst2::plugin::CanDo ) -> vst2::api::Supported {
		match can_do {
			vst2::plugin::CanDo::SendEvents |
			vst2::plugin::CanDo::SendMidiEvent =>
				vst2::api::Supported::Yes,
			_ =>
				vst2::api::Supported::No,
		}
	}

	fn process( &mut self, _: vst2::buffer::AudioBuffer<f32> ) {
		let bsize = HostCallbackExt::get_block_size( &self.host ) as f64;
		let (pos, rate, is_playing) = match HostCallbackExt::get_time( &self.host ) {
			Some( v ) => v,
			None      => return,
		};
		let mut shared = match self.shared.try_lock() {
			Ok ( v ) => v,
			Err( _ ) => return,
		};

		let mut events = Vec::new();

		if shared.changed || self.is_playing != is_playing {
			for ch in 0 .. 16 {
				// all notes off message.
				events.push( Self::event( &[ 0xb0 + ch, 0x7b, 0x00 ], 0 ) );
			}
			shared.changed = false;
		}

		if is_playing {
			let frame = |ev: &midi::Event| (ev.time * rate - pos).round();
			let ibgn = misc::bsearch_boundary( &shared.events, |ev| (frame( ev ), ev.prio) < (0.0,   i16::MIN) );
			let iend = misc::bsearch_boundary( &shared.events, |ev| (frame( ev ), ev.prio) < (bsize, i16::MIN) );
			for ev in shared.events[ibgn .. iend].iter() {
				events.push( Self::event( &ev.msg, frame( ev ) as i32 ) );
			}
		}

		self.host.process_events( events );
		self.is_playing = is_playing;
	}
}

impl Plugin {
	fn event<'a>( msg: &[u8], frame: i32 ) -> vst2::event::Event<'a> {
		vst2::event::Event::Midi{
			data: [ msg[0], msg[1], msg[2] ],
			delta_frames: frame,
			live: false,
			note_length: None,
			note_offset: None,
			detune: 0,
			note_off_velocity: 0,
		}
	}
}


plugin_main!( Plugin );
