use std::*;
use serde;
use serde_json;
use ws;
use memol::midi;


#[derive( Serialize, Deserialize, Debug )]
pub struct Event( f64, i16, u16, u32 );

impl From<midi::Event> for Event {
	fn from( ev: midi::Event ) -> Self {
		let buf =
			((ev.msg[0] as u32) <<  0) |
			((ev.msg[1] as u32) <<  8) |
			((ev.msg[2] as u32) << 16) |
			((ev.msg[3] as u32) << 24);
		Event( ev.time, ev.prio, ev.len, buf )
	}
}

impl Into<midi::Event> for Event {
	fn into( self ) -> midi::Event {
		midi::Event{
			time: self.0,
			prio: self.1,
			len:  self.2,
			msg: [
				(self.3 >>  0) as u8,
				(self.3 >>  8) as u8,
				(self.3 >> 16) as u8,
				(self.3 >> 24) as u8,
			],
		}
	}
}

#[derive( Serialize, Deserialize, Debug )]
#[serde( tag = "type" )]
pub enum Message {
	Success{ events: Vec<Event> },
	Failure{ message: String },
	Immediate{ events: Vec<Event> },
	Control{ is_playing: Option<bool>, location: Option<f64> },
	Status{ is_playing: bool, location: f64 },
}

pub struct Sender<T> {
	senders: sync::Arc<sync::Mutex<Vec<ws::Sender>>>,
	phantom: marker::PhantomData<T>,
}

impl<T> Clone for Sender<T> {
	fn clone( &self ) -> Self {
		Sender{
			senders: self.senders.clone(),
			phantom: marker::PhantomData,
		}
	}
}

impl<T: serde::Serialize> Sender<T> {
	pub fn send( &self, msg: &T ) -> Result<(), Box<error::Error>> {
		let buf = serde_json::to_string( msg )?;
		let senders = self.senders.lock().unwrap();
		for s in senders.iter() {
			s.send( buf.clone() )?;
		}
		Ok( () )
	}
}

pub struct Bus<T> {
	senders: sync::Arc<sync::Mutex<Vec<ws::Sender>>>,
	phantom: marker::PhantomData<T>,
}

impl<T: serde::Serialize + serde::de::DeserializeOwned> Bus<T> {
	pub fn new() -> Bus<T> {
		Bus{
			senders: sync::Arc::new( sync::Mutex::new( Vec::new() ) ),
			phantom: marker::PhantomData,
		}
	}

	pub fn create_sender( &self ) -> Sender<T> { 
		Sender{
			senders: self.senders.clone(),
			phantom: marker::PhantomData,
		}
	}

	pub fn listen<A: net::ToSocketAddrs + fmt::Debug, F: Fn( T )>( self, addr: A, f: F ) -> ws::Result<()> {
		let f = rc::Rc::new( f );
		ws::listen( addr, move |sender| {
			{
				let mut senders = self.senders.lock().unwrap();
				senders.push( sender.clone() );
			}
			let senders = self.senders.clone();
			let f = f.clone();
			move |buf: ws::Message| {
				{
					let senders = senders.lock().unwrap();
					for s in senders.iter() {
						if s.token() != sender.token() {
							s.send( buf.clone() ).ok();
						}
					}
				}
				match serde_json::from_slice( &buf.into_data() ) {
					Ok ( msg ) => Ok ( f( msg ) ),
					Err( err ) => Err( Box::new( err ).into() ),
				}
			}
		} )
	}

	pub fn connect<U: borrow::Borrow<str>, F: Fn( T )>( self, url: U, f: F ) -> ws::Result<()> {
		let f = rc::Rc::new( f );
		ws::connect( url, move |sender| {
			{
				let mut senders = self.senders.lock().unwrap();
				senders.push( sender );
			}
			let f = f.clone();
			move |buf: ws::Message| {
				match serde_json::from_slice( &buf.into_data() ) {
					Ok ( msg ) => Ok ( f( msg ) ),
					Err( err ) => Err( Box::new( err ).into() ),
				}
			}
		} )
	}
}
