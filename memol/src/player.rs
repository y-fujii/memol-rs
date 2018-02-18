// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use ratio;
use midi;


pub trait Player: Send {
	fn set_data( &self, events: Vec<midi::Event> );
	fn ports( &self ) -> io::Result<Vec<(String, bool)>>;
	fn connect( &self, port: &str ) -> io::Result<()>;
	fn disconnect( &self, port: &str ) -> io::Result<()>;
	fn play( &self ) -> io::Result<()>;
	fn stop( &self ) -> io::Result<()>;
	fn seek( &self, time: f64 ) -> io::Result<()>;
	fn location( &self ) -> ratio::Ratio;
	fn is_playing( &self ) -> bool;
}

pub struct DummyPlayer {}

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

	fn play( &self ) -> io::Result<()> {
		Ok( () )
	}

	fn stop( &self ) -> io::Result<()> {
		Ok( () )
	}

	fn seek( &self, _: f64 ) -> io::Result<()> {
		Ok( () )
	}

	fn location( &self ) -> ratio::Ratio {
		ratio::Ratio::zero()
	}

	fn is_playing( &self ) -> bool {
		false
	}
}

impl DummyPlayer {
	pub fn new() -> Box<DummyPlayer> {
		Box::new( DummyPlayer{} )
	}
}
