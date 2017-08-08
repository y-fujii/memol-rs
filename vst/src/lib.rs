// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
#[macro_use]
extern crate vst2;
extern crate memol;
use std::*;
use memol::{ misc, midi };
use vst2::host::Host;


struct HostCallbackExt {
	callback: Option<vst2::api::HostCallbackProc>,
	effect: *mut vst2::api::AEffect,
}

impl HostCallbackExt {
	fn callback( this: &vst2::plugin::HostCallback, op: vst2::host::OpCode, idx: i32, val: isize, ptr: *mut os::raw::c_void, opt: f32 ) -> isize {
		let this: &HostCallbackExt = unsafe { mem::transmute( this ) };
		this.callback.unwrap()( this.effect, op.into(), idx, val, ptr, opt )
	}

	fn get_time( this: &vst2::plugin::HostCallback ) -> Option<(f64, f64)> {
		let ptr = Self::callback( this, vst2::host::OpCode::GetTime, 0, 0, ptr::null_mut(), 0.0 );
		unsafe { (ptr as *const (f64, f64)).as_ref() }.cloned()
	}

	fn get_block_size( this: &vst2::plugin::HostCallback ) -> isize {
		Self::callback( this, vst2::host::OpCode::GetBlockSize, 0, 0, ptr::null_mut(), 0.0 )
	}
}

#[derive( Default )]
struct SharedData {
	events: Vec<midi::Event>,
}

#[derive( Default )]
struct Plugin {
	host: vst2::plugin::HostCallback,
	shared: sync::Arc<sync::Mutex<SharedData>>,
}

impl vst2::plugin::Plugin for Plugin {
	fn new( host: vst2::plugin::HostCallback ) -> Self {
		Self{
			host: host,
			.. Default::default()
		}
	}

    fn get_info( &self ) -> vst2::plugin::Info {
        vst2::plugin::Info{
            name: "memol".into(),
            unique_id: 271828182,
			inputs:  0,
			outputs: 0,
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
		let (spos, srate) = match HostCallbackExt::get_time( &self.host ) {
			Some( v ) => v,
			None      => return,
		};
		let shared = match self.shared.try_lock() {
			Ok ( v ) => v,
			Err( _ ) => return,
		};

		let ibgn = misc::bsearch_boundary( &shared.events, |ev| (ev.time * srate, ev.prio) < (spos,         i32::MIN) );
		let iend = misc::bsearch_boundary( &shared.events, |ev| (ev.time * srate, ev.prio) < (spos + bsize, i32::MIN) );
		let events = shared.events[ibgn .. iend].iter().map( |ev| vst2::event::Event::Midi{
			data: [ ev.msg[0], ev.msg[1], ev.msg[2] ],
			delta_frames: (ev.time * srate - spos).round() as i32,
			live: false,
			note_length: None,
			note_offset: None,
			detune: 0,
			note_off_velocity: 0,
		} ).collect();
		self.host.process_events( events );
	}
}

plugin_main!( Plugin );
