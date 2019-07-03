// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use memol::midi;


pub trait Player: Send {
	fn on_received_boxed( &mut self, _: Box<dyn 'static + Fn( &[midi::Event] ) + Send> );
	fn set_data( &mut self, _: &[midi::Event] );
	fn ports_from( &self ) -> io::Result<Vec<(String, bool)>>;
	fn connect_from( &self, _: &str ) -> io::Result<()>;
	fn disconnect_from( &self, _: &str ) -> io::Result<()>;
	fn ports_to( &self ) -> io::Result<Vec<(String, bool)>>;
	fn connect_to( &self, _: &str ) -> io::Result<()>;
	fn disconnect_to( &self, _: &str ) -> io::Result<()>;
	fn send( &self, _: &[midi::Event] );
	fn play( &self );
	fn stop( &self );
	fn seek( &self, _: f64 );
	fn status( &self ) -> (bool, f64);
	fn info( &self ) -> String;
}

pub trait PlayerExt {
	fn on_received<T: 'static + Fn( &[midi::Event] ) + Send>( &mut self, _: T );
}

impl<T: Player> PlayerExt for T {
	fn on_received<U: 'static + Fn( &[midi::Event] ) + Send>( &mut self, f: U ) {
		self.on_received_boxed( Box::new( f ) );
	}
}

impl PlayerExt for &mut dyn Player {
	fn on_received<T: 'static + Fn( &[midi::Event] ) + Send>( &mut self, f: T ) {
		self.on_received_boxed( Box::new( f ) );
	}
}

impl PlayerExt for Box<dyn Player> {
	fn on_received<T: 'static + Fn( &[midi::Event] ) + Send>( &mut self, f: T ) {
		self.on_received_boxed( Box::new( f ) );
	}
}

pub struct DummyPlayer {
	location: cell::Cell<f64>,
}

impl Player for DummyPlayer {
	fn on_received_boxed( &mut self, _: Box<dyn 'static + Fn( &[midi::Event] ) + Send> ) {
	}

	fn set_data( &mut self, _: &[midi::Event] ) {
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

	fn send( &self, _: &[midi::Event] ) {
	}

	fn play( &self ) {
	}

	fn stop( &self ) {
	}

	fn seek( &self, loc: f64 ) {
		self.location.set( loc );
	}

	fn status( &self ) -> (bool, f64) {
		(false, self.location.get())
	}

	fn info( &self ) -> String {
		String::new()
	}
}

impl DummyPlayer {
	pub fn new() -> Box<DummyPlayer> {
		Box::new( DummyPlayer{
			location: cell::Cell::new( 0.0 ),
		} )
	}
}
