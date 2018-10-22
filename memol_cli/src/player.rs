// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use memol::midi;


pub trait Player: Send {
	fn set_data( &self, Vec<midi::Event> );
	fn ports( &self ) -> io::Result<Vec<(String, bool)>>;
	fn connect( &self, &str ) -> io::Result<()>;
	fn disconnect( &self, &str ) -> io::Result<()>;
	fn send( &self, &[midi::Event] ) -> io::Result<()>;
	fn play( &self ) -> io::Result<()>;
	fn stop( &self ) -> io::Result<()>;
	fn seek( &self, f64 ) -> io::Result<()>;
	fn location( &self ) -> f64;
	fn is_playing( &self ) -> bool;
}

pub struct DummyPlayer {
	location: cell::Cell<f64>,
}

unsafe impl Send for DummyPlayer {}

impl Player for DummyPlayer {
	fn set_data( &self, _: Vec<midi::Event> ) {
	}

	fn ports( &self ) -> io::Result<Vec<(String, bool)>> {
		Ok( Vec::new() )
	}

	fn connect( &self, _: &str ) -> io::Result<()> {
		Ok( () )
	}

	fn disconnect( &self, _: &str ) -> io::Result<()> {
		Ok( () )
	}

	fn send( &self, _: &[midi::Event] ) -> io::Result<()> {
		Ok( () )
	}

	fn play( &self ) -> io::Result<()> {
		Ok( () )
	}

	fn stop( &self ) -> io::Result<()> {
		Ok( () )
	}

	fn seek( &self, loc: f64 ) -> io::Result<()> {
		self.location.set( loc );
		Ok( () )
	}

	fn location( &self ) -> f64 {
		self.location.get()
	}

	fn is_playing( &self ) -> bool {
		false
	}
}

impl DummyPlayer {
	pub fn new() -> Box<DummyPlayer> {
		Box::new( DummyPlayer{
			location: cell::Cell::new( 0.0 ),
		} )
	}
}
