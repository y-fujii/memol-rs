// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use clipboard::ClipboardProvider;
use memol::*;
use memol_cli::{ ipc, player };
use crate::compile_thread;


#[derive( Clone, Copy, PartialEq, Eq )]
pub enum DisplayMode {
	PianoRoll,
	Code,
}

pub struct Model {
	pub assembly: Assembly,
	pub events: Vec<midi::Event>,
	pub tempo: f64, // XXX
	pub path: path::PathBuf,
	pub code: String,
	pub mode: DisplayMode,
	pub channel: usize,
	pub follow: bool,
	pub autoplay: bool,
	pub text: Option<String>,
	pub bus_tx: ipc::Sender<ipc::Message>,
	pub player: Box<dyn player::Player>,
	pub compile_tx: sync::mpsc::Sender<compile_thread::Message>,
	pub use_sharp: bool,
	pub pedal: bool,
	pub on_notes: [bool; 128],
	pub copying_notes: Vec<i64>,
	pub clipboard: Option<cell::RefCell<clipboard::ClipboardContext>>,
}

impl Model {
	pub fn new( compile_tx: sync::mpsc::Sender<compile_thread::Message>, bus_tx: ipc::Sender<ipc::Message> ) -> Self {
		Model{
			assembly: Assembly::default(),
			events: Vec::new(),
			tempo: 1.0,
			path: path::PathBuf::new(),
			code: String::new(),
			mode: DisplayMode::PianoRoll,
			channel: 0,
			follow: true,
			autoplay: true,
			text: None,
			bus_tx: bus_tx,
			player: player::DummyPlayer::new(),
			compile_tx: compile_tx,
			use_sharp: false,
			pedal: false,
			on_notes: [false; 128],
			copying_notes: Vec::new(),
			clipboard: clipboard::ClipboardProvider::new().map( |e| cell::RefCell::new( e ) ).ok(),
		}
	}

	pub fn set_data( &mut self, path: path::PathBuf, code: String, asm: Assembly, evs: Vec<midi::Event> ) {
		self.path     = path;
		self.code     = code;
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

	pub fn handle_midi_inputs( &mut self ) {
		let mut events = Vec::new();
		self.player.recv( &mut events ).ok();

		for ev in events {
			match ev.msg[0] & 0xf0 {
				0x80 => {
					self.on_notes[ev.msg[1] as usize] = false;
				},
				0x90 => {
					self.on_notes[ev.msg[1] as usize] = true;
					if self.pedal {
						self.copying_notes.push( ev.msg[1] as i64 );
					}
				},
				0xb0 => {
					if ev.msg[1] == 64 {
						self.pedal = ev.msg[2] >= 64;
						if !self.pedal {
							self.copy_notes_to_clipboard();
						}
					}
				},
				_    => (),
			}
		}
	}

	pub fn note_on( &self, nn: u8 ) {
		let evs = [ midi::Event::new( 0.0, 1, &[ 0x90 + self.channel as u8, nn, 0x40 ] ) ];
		self.player.send( &evs ).ok();
		self.bus_tx.send( &ipc::Message::Immediate{
			events: evs.iter().map( |e| e.clone().into() ).collect()
		} ).unwrap();
	}

	pub fn note_off_all( &self ) {
		// all sound off.
		let evs = [ midi::Event::new( 0.0, 0, &[ 0xb0 + self.channel as u8, 0x78, 0x00 ] ) ];
		self.player.send( &evs ).ok();
		self.bus_tx.send( &ipc::Message::Immediate{
			events: evs.iter().map( |e| e.clone().into() ).collect()
		} ).unwrap();
	}

	pub fn note_symbol( &self, n: i64 ) -> &'static str {
		let syms = if self.use_sharp {
			[ "c", "c+", "d", "d+", "e", "f", "f+", "g", "g+", "a", "a+", "b" ]
		}
		else {
			[ "c", "d-", "d", "e-", "e", "f", "g-", "g", "a-", "a", "b-", "b" ]
		};
		syms[misc::imod( n, 12 ) as usize]
	}

	pub fn note_symbols( &self, notes: &[i64] ) -> String {
		let mut buf = String::new();
		let mut n0 = notes[0];
		for &n1 in notes.iter() {
			let sym = if n1 <= n0 { ">" } else { "<" };
			for _ in 0 .. (n1 - n0).abs() / 12 {
				buf.push_str( sym );
			}
			let sym = self.note_symbol( n1 );
			let sym = if n1 <= n0 { sym.to_lowercase() } else { sym.to_uppercase() };
			buf.push_str( &sym );
			n0 = n1;
		}
		buf
	}

	pub fn copy_notes_to_clipboard( &mut self ) {
		if self.copying_notes.is_empty() {
			return;
		}
		if let Some( ref clipboard ) = self.clipboard {
			clipboard.borrow_mut().set_contents( self.note_symbols( &self.copying_notes ) ).ok();
		}
		self.copying_notes.clear();
	}

	pub fn generate_smf( &self ) -> io::Result<()> {
		let smf = self.path.with_extension( "mid" );
		let mut buf = io::BufWriter::new( fs::File::create( smf )? );
		memol::smf::write_smf( &mut buf, &self.events, 480 )?;
		Ok( () )
	}
}
