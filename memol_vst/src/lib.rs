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

struct SharedData {
	locked: sync::Mutex<LockedData>,
	immediate_send: crossbeam_queue::ArrayQueue<midi::Event>,
	immediate_recv: crossbeam_queue::ArrayQueue<midi::Event>,
	playing: atomic::AtomicBool,
	location: atomic::AtomicIsize,
	condvar: sync::Condvar,
	exiting: atomic::AtomicBool,
	stream: sync::Mutex<Option<net::TcpStream>>,
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
		self.shared.exiting.store( true, atomic::Ordering::SeqCst );
		self.shared.condvar.notify_all();
		if let Some( stream ) = self.shared.stream.lock().unwrap().take() {
			stream.shutdown( net::Shutdown::Both ).ok();
		}
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
				location: atomic::AtomicIsize::new( 0 ),
				condvar: sync::Condvar::new(),
				exiting: atomic::AtomicBool::new( false ),
				stream: sync::Mutex::new( None ),
			} ),
			playing: false,
			location: 0,
		}
	}
}

impl vst::plugin::Plugin for Plugin {
	fn new( host: vst::plugin::HostCallback ) -> Self {
		let mut this = Plugin::default();
		this.host = host;

		// XXX: finalization is not complete.
		let shared = this.shared.clone();
		this.handle = Some( thread::spawn( move || {
			while !shared.exiting.load( atomic::Ordering::SeqCst ) {
				thread::sleep( time::Duration::from_secs( 1 ) );

				let addr = net::SocketAddr::new( net::IpAddr::V4( net::Ipv4Addr::new( 127, 0, 0, 1 ) ), 27182 );
				let stream = match net::TcpStream::connect_timeout( &addr, time::Duration::from_secs( 3 ) ) {
					Ok ( s ) => s,
					Err( _ ) => continue,
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
						while !shared.exiting.load( atomic::Ordering::SeqCst ) {
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
						let mutex = sync::Mutex::new( () );
						let mut lock = mutex.lock().unwrap();
						while !shared.exiting.load( atomic::Ordering::SeqCst ) {
							let msg = (
								shared.playing .load( atomic::Ordering::SeqCst ),
								shared.location.load( atomic::Ordering::SeqCst ),
							);
							match stream.write_all( &bincode::serialize( &msg ).unwrap() ) {
								Ok ( _ ) => (),
								Err( _ ) => break,
							}
							if shared.exiting.load( atomic::Ordering::SeqCst ) {
								break;
							}
							lock = shared.condvar.wait( lock ).unwrap();
						}
						stream.shutdown( net::Shutdown::Both ).ok();
					}
				};

				*shared.stream.lock().unwrap() = Some( stream );
				let handle = thread::spawn( reader );
				writer();
				handle.join().ok();
				*shared.stream.lock().unwrap() = None;
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

		self.shared.location.store( location, atomic::Ordering::SeqCst );
		self.shared.playing .store( playing , atomic::Ordering::SeqCst );
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
