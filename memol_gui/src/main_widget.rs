// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use imutil;
use renderer;
use piano_roll;
use compile_thread;
use transport_widget;
use std::*;
use memol::*;
use memol_cli::{ ipc, player };


const JACK_FRAME_WAIT: i32 = 12;

pub struct MainWidget {
	pub player: Box<player::Player>,
	pub text: Option<String>,
	pub wallpaper: Option<renderer::Texture>,
	compile_tx: sync::mpsc::Sender<compile_thread::Message>,
	bus_tx: ipc::Sender<ipc::Message>,
	path: path::PathBuf,
	assembly: Assembly,
	events: Vec<midi::Event>,
	tempo: f64, // XXX
	transport: transport_widget::TransportWidget,
	piano_roll: piano_roll::PianoRoll,
}

impl MainWidget {
	pub fn new( compile_tx: sync::mpsc::Sender<compile_thread::Message>, bus_tx: ipc::Sender<ipc::Message> ) -> Self {
		MainWidget{
			bus_tx: bus_tx,
			compile_tx: compile_tx,
			path: path::PathBuf::new(),
			assembly: Assembly::default(),
			events: Vec::new(),
			tempo: 1.0,
			text: None,
			player: player::DummyPlayer::new(),
			transport: transport_widget::TransportWidget::new(),
			piano_roll: piano_roll::PianoRoll::new(),
			wallpaper: None,
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
		if self.transport.autoplay && !self.player.is_playing() {
			self.player.seek( bgn ).ok();
			self.player.play().ok();
		}
	}

	pub fn draw( &mut self ) -> i32 {
		let mut events = Vec::new();
		self.player.recv( &mut events ).ok();
		self.piano_roll.handle_midi_inputs( events.iter() );
		let n = unsafe { self.draw_widget() };
		self.handle_events();
		n
	}

	unsafe fn draw_widget( &mut self ) -> i32 {
		use imgui::*;

		let is_playing = self.player.is_playing();
		let location   = self.player.location();

		if let Some( ref text ) = self.text {
			imutil::message_dialog( "Message", text );
		}

		let channels: Vec<_> = self.assembly.channels.iter().map( |&(i, _)| i ).collect();
		let changed = self.transport.draw( &*self.player, &channels );

		PushStyleColor( ImGuiCol_WindowBg as i32, 0xffff_ffff );
		imutil::root_begin( 0 );
			let size = GetWindowSize();
			if let Some( &(_, ref ch) ) = self.assembly.channels
				.iter().filter( |&&(i, _)| i == self.transport.channel ).next()
			{
				self.piano_roll.draw(
					&ch.score, self.assembly.len.to_float() as f32,
					(location * self.tempo) as f32,
					is_playing && self.transport.follow, size,
				);
			}
			if let Some( ref wallpaper ) = self.wallpaper {
				let scale = f32::max( size.x / wallpaper.size.0 as f32, size.y / wallpaper.size.1 as f32 );
				let wsize = scale * ImVec2::new( wallpaper.size.0 as f32, wallpaper.size.1 as f32 );
				let v0 = GetWindowPos() + self.piano_roll.scroll_ratio * (size - wsize);
				(*GetWindowDrawList()).AddImage(
					wallpaper.id as _, &v0, &(v0 + wsize), &ImVec2::zero(), &ImVec2::new( 1.0, 1.0 ), 0xffff_ffff,
				);
			}
		imutil::root_end();
		PopStyleColor( 1 );

		if changed { JACK_FRAME_WAIT } else if is_playing { 1 } else { 0 }
	}

	fn handle_events( &mut self ) {
		for ev in self.transport.events.drain( .. ) {
			match ev {
				transport_widget::Event::GenerateSmf => {
					if let Err( e ) = Self::generate_smf( &self.path, &self.events ) {
						self.text = Some( format!( "{}", e ) );
					}
				},
				transport_widget::Event::Refresh => {
					self.compile_tx.send( compile_thread::Message::Refresh ).unwrap();
				},
			}
		}

		for ev in self.piano_roll.events.drain( .. ) {
			match ev {
				piano_roll::Event::Seek( loc ) => {
					self.player.seek( f64::max( loc as f64, 0.0 ) / self.tempo ).ok();
				},
				piano_roll::Event::NoteOn( n ) => {
					let evs = [ midi::Event::new( 0.0, 1, &[ 0x90 + self.transport.channel as u8, n as u8, 0x40 ] ) ];
					self.player.send( &evs ).ok();
					self.bus_tx.send( &ipc::Message::Immediate{
						events: evs.iter().map( |e| e.clone().into() ).collect()
					} ).unwrap();
				},
				piano_roll::Event::NoteClear => {
					// all sound off.
					let evs = [ midi::Event::new( 0.0, 0, &[ 0xb0 + self.transport.channel as u8, 0x78, 0x00 ] ) ];
					self.player.send( &evs ).ok();
					self.bus_tx.send( &ipc::Message::Immediate{
						events: evs.iter().map( |e| e.clone().into() ).collect()
					} ).unwrap();
				},
			}
		}
	}

	fn generate_smf( path: &path::PathBuf, events: &Vec<midi::Event>  ) -> io::Result<()> {
		let smf = path.with_extension( "mid" );
		let mut buf = io::BufWriter::new( fs::File::create( smf )? );
		memol::smf::write_smf( &mut buf, &events, 480 )?;
		Ok( () )
	}
}
