// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use std::io::Write;
use byteorder::{ LittleEndian, WriteBytesExt };
use memol::midi;
use crate::player;


struct Shared {
	events: Vec<midi::Event>,
	senders: Vec<sync::mpsc::Sender<Vec<u8>>>,
}

pub struct Player {
	shared: sync::Arc<sync::Mutex<Shared>>,
}

impl player::Player for Player {
	fn on_received_boxed( &mut self, _: Box<dyn 'static + Fn() + Send> ) {
	}

	fn set_data( &mut self, events: Vec<midi::Event> ) {
		let msg = Self::encode( &events );
		let mut shared = self.shared.lock().unwrap();
		shared.events = events;
		for sender in shared.senders.iter() {
			sender.send( msg.clone() ).ok();
		}
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

	fn location( &self ) -> f64 {
		0.0
	}

	fn is_playing( &self ) -> bool {
		false
	}

	fn status( &self ) -> String {
		let cnt = self.shared.lock().unwrap().senders.len();
		format!( "{} VST plugin(s) are connected.", cnt )
	}
}

impl Player {
	pub fn new<T: net::ToSocketAddrs>( addr: T ) -> io::Result<Box<Self>> {
		let shared = sync::Arc::new( sync::Mutex::new( Shared{
			events: Vec::new(),
			senders: Vec::new(),
		} ) );

		let listener = net::TcpListener::bind( addr )?;
		thread::spawn( {
			let shared = shared.clone();
			move || {
				for stream in listener.incoming() {
					let mut stream = match stream {
						Ok ( s ) => s,
						Err( _ ) => continue,
					};
					thread::spawn( {
						let shared = shared.clone();
						move || {
							stream.set_nodelay( true ).ok();
							let (sender, receiver) = sync::mpsc::channel();
							let msg = {
								let mut shared = shared.lock().unwrap();
								shared.senders.push( sender );
								Self::encode( &shared.events )
							};
							if let Err( _ ) = stream.write_all( &msg ) {
								return;
							}
							loop {
								let msg = receiver.recv().unwrap();
								if let Err( _ ) = stream.write_all( &msg ) {
									return;
								}
							}
						}
					} );
				}
			}
		} );

		Ok( Box::new( Player{
			shared: shared,
		} ) )
	}

	fn encode( events: &Vec<midi::Event> ) -> Vec<u8> {
		let mut buf = Vec::new();
		buf.write_u64::<LittleEndian>( 0 ).unwrap(); // type.
		buf.write_u64::<LittleEndian>( 16 * events.len() as u64 ).unwrap();
		for ev in events.iter() {
			buf.write_f64::<LittleEndian>( ev.time ).unwrap();
			buf.write_i16::<LittleEndian>( ev.prio ).unwrap();
			buf.write_u16::<LittleEndian>( ev.len ).unwrap();
			buf.write_all( &ev.msg ).unwrap();
		}
		buf
	}
}
