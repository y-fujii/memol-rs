// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use std::io::Write;
use std::net::ToSocketAddrs;
use vst::plugin_main;
use vst::host::Host;
use memol::{ misc, midi };
use memol_cli::player_net;
mod events;


const BUFFER_LEN: usize = 65536;

struct LockedData {
	events: Vec<midi::Event>,
	changed: bool,
	immediate_send: Vec<midi::Event>,
	immediate_recv: Vec<midi::Event>,
	playing: bool,
	seconds: f64,
}

struct Exiter {
	exiting: bool,
	stream: Option<net::TcpStream>,
}

struct SharedData {
	locked: sync::Mutex<LockedData>,
	condvar: sync::Condvar,
	exiter: sync::Mutex<Exiter>,
}

struct Plugin {
	host: vst::plugin::HostCallback,
	buffer: events::EventBuffer,
	handle: Option<thread::JoinHandle<()>>,
	events: Vec<midi::Event>,
	immediate_send: Vec<midi::Event>,
	immediate_recv: Vec<midi::Event>,
	playing: bool,
	location: isize,
	shared: sync::Arc<SharedData>,
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
			events: Vec::new(),
			immediate_send: Vec::with_capacity( BUFFER_LEN ),
			immediate_recv: Vec::with_capacity( BUFFER_LEN ),
			playing: false,
			location: 0,
			shared: sync::Arc::new( SharedData{
				locked: sync::Mutex::new( LockedData{
					events: Vec::new(),
					changed: false,
					immediate_send: Vec::new(),
					immediate_recv: Vec::with_capacity( BUFFER_LEN ),
					playing: false,
					seconds: 0.0,
				} ),
				condvar: sync::Condvar::new(),
				exiter: sync::Mutex::new( Exiter{
					exiting: false,
					stream: None,
				} ),
			} ),
		}
	}
}

impl vst::plugin::Plugin for Plugin {
	fn new( host: vst::plugin::HostCallback ) -> Self {
		let localhosts = vec![
			net::SocketAddr::new( net::IpAddr::V6( net::Ipv6Addr::LOCALHOST ), 27182 ),
			net::SocketAddr::new( net::IpAddr::V4( net::Ipv4Addr::LOCALHOST ), 27182 ),
		];
		let addrs = match env::var( "MEMOL_VST_ADDR" ) {
			Ok( e ) => match e.to_socket_addrs() {
				Ok ( e ) => e.collect(),
				Err( _ ) => localhosts,
			},
			Err( _ ) => localhosts,
		};

		let mut this = Plugin::default();
		this.host = host;

		// XXX: finalization is not complete.
		let shared = this.shared.clone();
		this.handle = Some( thread::spawn( move || {
			loop {
				thread::sleep( time::Duration::from_secs( 1 ) );
				let stream = {
					let mut exiter = shared.exiter.lock().unwrap();
					if exiter.exiting {
						break;
					}
					let mut stream = None;
					for addr in addrs.iter() {
						match net::TcpStream::connect_timeout( addr, time::Duration::from_secs( 3 ) ) {
							Ok( s ) => {
								stream = Some( s );
								break;
							},
							Err( _ ) => continue,
						};
					}
					let stream = match stream {
						Some( s ) => s,
						None      => continue,
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
							let msg = match player_net::StcMessage::deserialize_from( &mut stream ) {
								Ok ( e ) => e,
								Err( _ ) => break,
							};
							match msg {
								player_net::StcMessage::Data( evs ) => {
									let mut locked = shared.locked.lock().unwrap();
									locked.events = evs;
									locked.changed = true;
								},
								player_net::StcMessage::Immediate( evs ) => {
									let mut locked = shared.locked.lock().unwrap();
									locked.immediate_send.extend( evs );
								},
							}
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
							let mut evs = Vec::with_capacity( BUFFER_LEN );
							let (playing, seconds);
							{
								let mut locked = shared.locked.lock().unwrap();
								mem::swap( &mut evs, &mut locked.immediate_recv );
								playing = locked.playing;
								seconds = locked.seconds;
							}

							if evs.len() > 0 {
								let msg = player_net::CtsMessage::Immediate( evs );
								match stream.write_all( &bincode::serialize( &msg ).unwrap() ) {
									Ok ( _ ) => (),
									Err( _ ) => break,
								}
							}

							let msg = player_net::CtsMessage::Status( playing, seconds );
							match stream.write_all( &bincode::serialize( &msg ).unwrap() ) {
								Ok ( _ ) => (),
								Err( _ ) => break,
							}

							let exiter = shared.exiter.lock().unwrap();
							if exiter.exiting {
								break;
							}
							if shared.condvar.wait( exiter ).unwrap().exiting {
								break;
							}
						}
						stream.shutdown( net::Shutdown::Both ).ok();
					}
				};

				let handle = thread::spawn( reader );
				writer();
				handle.join().ok();
				shared.exiter.lock().unwrap().stream = None;
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
		// workaround for Cubase.
		let location_fixed = if (self.location - location).abs() < 4 { self.location } else { location };
		let playing = info.flags & vst::api::TimeInfoFlags::TRANSPORT_PLAYING.bits() != 0;

		let mut changed = false;
		if let Ok( mut locked ) = self.shared.locked.try_lock() {
			if mem::replace( &mut locked.changed, false ) {
				mem::swap( &mut self.events, &mut locked.events );
				changed = true;
			}

			locked.immediate_recv.extend( self.immediate_recv.drain( .. ) );
			self.immediate_send.extend( locked.immediate_send.drain( .. ) );

			let seconds = location as f64 / info.sample_rate;
			if playing != locked.playing || seconds != locked.seconds || locked.immediate_recv.len() > 0 {
				// XXX: may fail to wake the network thread.
				self.shared.condvar.notify_one();
			}
			locked.playing = playing;
			locked.seconds = seconds;
		}

		if changed || playing != self.playing || (playing && location_fixed != self.location) {
			for ch in 0 .. 16 {
				// all sound off.
				self.buffer.push( &[ 0xb0 + ch, 0x78, 0x00 ], 0 );
				// reset all controllers.
				self.buffer.push( &[ 0xb0 + ch, 0x79, 0x00 ], 0 );
			}
		}

		// XXX: add delay.
		for ev in self.immediate_send.drain( .. ) {
			self.buffer.push( &ev.msg, 0 );
		}

		if playing {
			let frame = |ev: &midi::Event| (ev.time * info.sample_rate).round() as isize;
			let ibgn = misc::bsearch_boundary( &self.events, |ev| frame( ev ) < location_fixed  );
			let iend = misc::bsearch_boundary( &self.events, |ev| frame( ev ) < location + size );
			for ev in self.events[ibgn .. iend].iter() {
				let i = cmp::max( frame( ev ) - location, 0 );
				self.buffer.push( &ev.msg, i as i32 );
			}
		}

		self.host.process_events( self.buffer.events() );
		self.playing = playing;
		self.location = location + if playing { size } else { 0 };
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
				0xb0 =>  0,
				_    => continue,
			};
			self.immediate_recv.push( midi::Event::new( 0.0, prio, &ev.data ) );
		}
	}
}

plugin_main!( Plugin );
