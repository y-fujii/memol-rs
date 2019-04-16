// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use std::io::Write;
use memol::midi;
use crate::player;


pub enum CtsMessage {
	Status( bool, f64 ),
	Immediate( Vec<midi::Event> ),
}

pub enum StcMessage {
	Data( Vec<midi::Event> ),
	Immediate( Vec<midi::Event> ),
}

struct CtsShared {
	playing: bool,
	location: f64,
	arrival: time::SystemTime,
}

struct StcShared {
	events: Vec<midi::Event>,
	senders: Vec<sync::mpsc::Sender<Vec<u8>>>,
}

pub struct Player {
	cts_shared: sync::Arc<sync::Mutex<CtsShared>>,
	stc_shared: sync::Arc<sync::Mutex<StcShared>>,
}

impl player::Player for Player {
	fn on_received_boxed( &mut self, _: Box<dyn 'static + Fn() + Send> ) {
	}

	fn set_data( &mut self, events: Vec<midi::Event> ) {
		let msg = bincode::serialize( &events ).unwrap();
		let mut shared = self.stc_shared.lock().unwrap();
		shared.events = events;
		shared.senders.retain( |s| s.send( msg.clone() ).is_ok() );
	}

	fn ports_from( &self ) -> io::Result<Vec<(String, bool)>> {
		Ok( Vec::new() )
	}

	fn connect_from( &self, _: &str ) -> io::Result<()> {
		Ok( () )
	}

	fn disconnect_from( &self, _: &str ) -> io::Result<()> {
		Ok( () )
	}

	fn ports_to( &self ) -> io::Result<Vec<(String, bool)>> {
		Ok( Vec::new() )
	}

	fn connect_to( &self, _: &str ) -> io::Result<()> {
		Ok( () )
	}

	fn disconnect_to( &self, _: &str ) -> io::Result<()> {
		Ok( () )
	}

	fn send( &self, _: &[midi::Event] ) -> io::Result<()> {
		Ok( () )
	}

	fn recv( &self, _: &mut Vec<midi::Event> ) -> io::Result<()> {
		Ok( () )
	}

	fn play( &self ) -> io::Result<()> {
		Ok( () )
	}

	fn stop( &self ) -> io::Result<()> {
		Ok( () )
	}

	fn seek( &self, _: f64 ) -> io::Result<()> {
		Ok( () )
	}

	fn status( &self ) -> (bool, f64) {
		let shared = self.cts_shared.lock().unwrap();
		let playing = shared.playing;
		let delta = shared.arrival.elapsed().unwrap_or( time::Duration::new( 0, 0 ) );
		let loc = if playing {
			1e-9 * delta.subsec_nanos() as f64 + delta.as_secs() as f64 + shared.location
		}
		else {
			shared.location
		};
		(playing, loc)
	}

	fn info( &self ) -> String {
		let cnt = self.stc_shared.lock().unwrap().senders.len();
		format!( "{} VST plugin(s) are connected.", cnt )
	}
}

impl Player {
	// XXX: finalization.
	pub fn new<T: net::ToSocketAddrs>( addr: T ) -> io::Result<Box<Self>> {
		let cts_shared = sync::Arc::new( sync::Mutex::new( CtsShared{
			playing: false,
			location: 0.0,
			arrival: time::SystemTime::UNIX_EPOCH,
		} ) );
		let stc_shared = sync::Arc::new( sync::Mutex::new( StcShared{
			events: Vec::new(),
			senders: Vec::new(),
		} ) );

		let listener = net::TcpListener::bind( addr )?;
		thread::spawn( {
			let cts_shared = cts_shared.clone();
			let stc_shared = stc_shared.clone();
			move || {
				for stream in listener.incoming() {
					let stream = match stream {
						Ok ( s ) => s,
						Err( _ ) => continue,
					};
					stream.set_nodelay( true ).ok();

					// reader thread.
					thread::spawn( {
						let shared = cts_shared.clone();
						let mut stream = match stream.try_clone() {
							Ok ( s ) => s,
							Err( _ ) => continue,
						};
						move || move || -> Result<(), Box<dyn error::Error>> {
							loop {
								let msg: (bool, f64) = bincode::deserialize_from( &mut stream )?;
								let now = time::SystemTime::now();
								let mut shared = shared.lock().unwrap();
								shared.playing  = msg.0;
								shared.location = msg.1;
								shared.arrival  = now;
							}
						}().ok()
					} );

					// writer thread.
					thread::spawn( {
						let shared = stc_shared.clone();
						let mut stream = match stream.try_clone() {
							Ok ( s ) => s,
							Err( _ ) => continue,
						};
						move || move || -> Result<(), Box<dyn error::Error>> {
							let (sender, receiver) = sync::mpsc::channel();
							let msg = {
								let mut shared = shared.lock().unwrap();
								shared.senders.push( sender );
								bincode::serialize( &shared.events ).unwrap()
							};
							stream.write_all( &msg )?;
							loop {
								let msg = receiver.recv()?;
								stream.write_all( &msg )?;
							}
						}().ok()
					} );
				}
			}
		} );

		Ok( Box::new( Player{
			cts_shared: cts_shared,
			stc_shared: stc_shared,
		} ) )
	}
}
