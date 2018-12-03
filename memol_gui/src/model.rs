// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use compile_thread;
use std::*;
use memol::*;
use memol_cli::{ ipc, player };


pub struct Model {
	pub assembly: Assembly,
	pub events: Vec<midi::Event>,
	pub tempo: f64, // XXX
	pub path: path::PathBuf,
	pub channel: usize,
	pub follow: bool,
	pub autoplay: bool,
	pub text: Option<String>,
	pub bus_tx: ipc::Sender<ipc::Message>,
	pub player: Box<player::Player>,
	pub compile_tx: sync::mpsc::Sender<compile_thread::Message>,
}

impl Model {
	pub fn new( compile_tx: sync::mpsc::Sender<compile_thread::Message>, bus_tx: ipc::Sender<ipc::Message> ) -> Self {
		Model{
			assembly: Assembly::default(),
			events: Vec::new(),
			tempo: 1.0,
			path: path::PathBuf::new(),
			channel: 0,
			follow: true,
			autoplay: true,
			text: None,
			bus_tx: bus_tx,
			player: player::DummyPlayer::new(),
			compile_tx: compile_tx,
		}
	}

	pub fn set_data( &mut self, path: path::PathBuf, asm: Assembly, evs: Vec<midi::Event> ) {
		self.path     = path;
		self.assembly = asm;
		self.events   = evs;
		self.text     = None;
		// XXX
		let rng = random::Generator::new();
		let evaluator = generator::Evaluator::new( &rng );
		self.tempo = evaluator.eval( &self.assembly.tempo, ratio::Ratio::zero() );

		let bgn = match self.events.get( 0 ) {
			Some( ev ) => ev.time.max( 0.0 ),
			None       => 0.0,
		};
		self.player.set_data( self.events.clone() );
		if self.autoplay && !self.player.is_playing() {
			self.player.seek( bgn ).ok();
			self.player.play().ok();
		}
	}

	pub fn generate_smf( &self ) -> io::Result<()> {
		let smf = self.path.with_extension( "mid" );
		let mut buf = io::BufWriter::new( fs::File::create( smf )? );
		memol::smf::write_smf( &mut buf, &self.events, 480 )?;
		Ok( () )
	}

	pub fn note_on( &self, nn: u8 ) {
		let evs = [ midi::Event::new( 0.0, 1, &[ 0x90 + self.channel as u8, nn, 0x40 ] ) ];
		self.player.send( &evs ).ok();
		self.bus_tx.send( &ipc::Message::Immediate{
			events: evs.iter().map( |e| e.clone().into() ).collect()
		} ).unwrap();
	}

	pub fn note_clear( &self ) {
		// all sound off.
		let evs = [ midi::Event::new( 0.0, 0, &[ 0xb0 + self.channel as u8, 0x78, 0x00 ] ) ];
		self.player.send( &evs ).ok();
		self.bus_tx.send( &ipc::Message::Immediate{
			events: evs.iter().map( |e| e.clone().into() ).collect()
		} ).unwrap();
	}
}
