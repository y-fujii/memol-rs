// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use std::io::Write;
use std::sync::atomic;
use vst::plugin_main;
use vst::host::Host;
use memol::{ misc, midi };
mod events;


struct LockedData {
	events: Vec<midi::Event>,
	changed: bool,
}

struct Exiter {
	exiting: bool,
	stream: Option<net::TcpStream>,
}

struct SharedData {
	locked: sync::Mutex<LockedData>,
	immediate_send: crossbeam_queue::ArrayQueue<midi::Event>,
	immediate_recv: crossbeam_queue::ArrayQueue<midi::Event>,
	playing: atomic::AtomicBool,
	location: atomic::AtomicUsize,
	condvar: sync::Condvar,
	exiter: sync::Mutex<Exiter>,
}

struct Plugin {
	host: vst::plugin::HostCallback,
	buffer: events::EventBuffer,
	handle: Option<thread::JoinHandle<()>>,
	shared: sync::Arc<SharedData>,
	playing: bool,
	location: isize,
}

impl Drop for Plugin {
	fn drop( &mut self ) {
		{
			let mut exiter = self.shared.exiter.lock().unwrap();
			exiter.exiting = true;
			if let Some( stream ) = exiter.stream.take() {
				stream.shutdown( net::Shutdown::Both ).ok();
			}
		}
		self.shared.condvar.notify_all();
		if let Some( handle ) = self.handle.take() {
			handle.join().ok();
		}
	}
}

impl default::Default for Plugin {
	fn default() -> Self {
		Plugin{
			host: vst::plugin::HostCallback::default(),
			buffer: events::EventBuffer::new(),
			handle: None,
			shared: sync::Arc::new( SharedData{
				locked: sync::Mutex::new( LockedData{
					events: Vec::new(),
					changed: false,
				} ),
				immediate_send: crossbeam_queue::ArrayQueue::new( 4096 ),
				immediate_recv: crossbeam_queue::ArrayQueue::new( 4096 ),
				playing: atomic::AtomicBool::new( false ),
				location: atomic::AtomicUsize::new( 0.0f64.to_bits() as usize ),
				condvar: sync::Condvar::new(),
				exiter: sync::Mutex::new( Exiter{
					exiting: false,
					stream: None,
				} ),
			} ),
			playing: false,
			location: 0,
		}
	}
}

impl vst::plugin::Plugin for Plugin {
	fn new( host: vst::plugin::HostCallback ) -> Self {
		let address = net::SocketAddr::new( net::IpAddr::V4( net::Ipv4Addr::new( 127, 0, 0, 1 ) ), 27182 );

		let mut this = Plugin::default();
		this.host = host;

		// XXX: finalization is not complete.
		let shared = this.shared.clone();
		this.handle = Some( thread::spawn( move || {
			loop {
				let stream = {
					let mut exiter = shared.exiter.lock().unwrap();
					if exiter.exiting {
						break;
					}
					let stream = match net::TcpStream::connect_timeout( &address, time::Duration::from_secs( 3 ) ) {
						Ok ( s ) => s,
						Err( _ ) => continue,
					};
					exiter.stream = Some( match stream.try_clone() {
						Ok ( s ) => s,
						Err( _ ) => continue,
					} );
					stream
				};
				stream.set_nodelay( true ).ok();

				let reader = {
					let shared = shared.clone();
					let stream = match stream.try_clone() {
						Ok ( s ) => s,
						Err( _ ) => continue,
					};
					move || {
						let mut stream = io::BufReader::new( stream );
						while !shared.exiter.lock().unwrap().exiting {
							let events = match bincode::deserialize_from( &mut stream ) {
								Ok ( e ) => e,
								Err( _ ) => break,
							};
							let mut locked = shared.locked.lock().unwrap();
							locked.events = events;
							locked.changed = true;
						}
						stream.get_ref().shutdown( net::Shutdown::Both ).ok();
					}
				};

				let mut writer = {
					let shared = shared.clone();
					let mut stream = match stream.try_clone() {
						Ok ( s ) => s,
						Err( _ ) => continue,
					};
					move || {
						loop {
							let msg = (
								shared.playing .load( atomic::Ordering::SeqCst ),
								shared.location.load( atomic::Ordering::SeqCst ),
							);
							match stream.write_all( &bincode::serialize( &msg ).unwrap() ) {
								Ok ( _ ) => (),
								Err( _ ) => break,
							}

							let exiter = shared.exiter.lock().unwrap();
							if exiter.exiting {
								break;
							}
							drop( shared.condvar.wait( exiter ).unwrap() );
						}
						stream.shutdown( net::Shutdown::Both ).ok();
					}
				};

				let handle = thread::spawn( reader );
				writer();
				handle.join().ok();
				shared.exiter.lock().unwrap().stream = None;

				thread::sleep( time::Duration::from_secs( 1 ) );
			}
		} ) );

		this
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
			vst::plugin::CanDo::ReceiveEvents |
			vst::plugin::CanDo::ReceiveMidiEvent |
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
		let location = info.sample_pos.round() as isize;
		let playing  = info.flags & vst::api::flags::TRANSPORT_PLAYING.bits() != 0;

		let loc_sfu = (location as f64 / info.sample_rate).to_bits() as usize;
		self.shared.location.store( loc_sfu, atomic::Ordering::SeqCst );
		self.shared.playing .store( playing, atomic::Ordering::SeqCst );
		self.shared.condvar.notify_one();

		let mut locked = match self.shared.locked.try_lock() {
			Ok ( v ) => v,
			Err( _ ) => return,
		};

		if locked.changed || playing != self.playing || (playing && location != self.location) {
			for ch in 0 .. 16 {
				// all sound off.
				self.buffer.push( &[ 0xb0 + ch, 0x78, 0x00 ], 0 );
				// reset all controllers.
				self.buffer.push( &[ 0xb0 + ch, 0x79, 0x00 ], 0 );
			}
			locked.changed = false;
		}

		// XXX: add delay.
		while let Ok( ev ) = self.shared.immediate_send.pop() {
			self.buffer.push( &ev.msg, 0 );
		}

		if playing {
			let frame = |ev: &midi::Event| (ev.time * info.sample_rate).round() as isize - location;
			let ibgn = misc::bsearch_boundary( &locked.events, |ev| frame( ev ) < 0    );
			let iend = misc::bsearch_boundary( &locked.events, |ev| frame( ev ) < size );
			for ev in locked.events[ibgn .. iend].iter() {
				self.buffer.push( &ev.msg, frame( ev ) as i32 );
			}
		}

		self.host.process_events( self.buffer.events() );
		self.location = location + size;
		self.playing  = playing;
	}

	fn process_events( &mut self, events: &vst::api::Events ) {
		for ev in events.events() {
			let ev = match ev {
				vst::event::Event::Midi( e ) => e,
				_                            => continue,
			};
			let prio = match ev.data[0] & 0xf0 {
				0x80 => -1,
				0x90 =>  1,
				_    => continue,
			};
			self.shared.immediate_recv.push( midi::Event{
				time: 0.0, // XXX
				prio: prio,
				len: 3,
				msg: [ ev.data[0], ev.data[1], ev.data[2], 0 ],
			} ).ok();
		}
		if !self.shared.immediate_recv.is_empty() {
			self.shared.condvar.notify_one();
		}
	}
}

plugin_main!( Plugin );
