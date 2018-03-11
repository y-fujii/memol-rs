// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
#![windows_subsystem = "windows"]
extern crate getopts;
extern crate gl;
extern crate glutin;
extern crate image;
extern crate memol;
extern crate memol_cli;
#[macro_use]
mod imutil;
mod imgui;
mod renderer;
mod window;
mod pianoroll;
use std::*;
use memol::*;
use memol_cli::{ notify, ipc, player, player_jack };
use memol_cli::player::Player;


const JACK_FRAME_WAIT: i32 = 12;

enum UiMessage {
	Data( path::PathBuf, Assembly, Vec<midi::Event> ),
	Text( String ),
	Player( Box<player::Player> ),
}

enum CompilerMessage {
    File( path::PathBuf ),
    Refresh,
}

struct Ui {
	compile_tx: sync::mpsc::Sender<CompilerMessage>,
	path: path::PathBuf,
	assembly: Assembly,
	events: Vec<midi::Event>,
	tempo: f64, // XXX
	text: Option<String>,
	player: Box<player::Player>,
	piano_roll: pianoroll::PianoRoll,
	channel: usize,
	follow: bool,
	autoplay: bool,
	ports: Vec<(String, bool)>,
	wallpaper: Option<renderer::Texture>,
}

impl window::Ui<UiMessage> for Ui {
	fn on_draw( &mut self ) -> i32 {
		unsafe { self.draw_all() }
	}

	fn on_file_dropped( &mut self, path: &path::PathBuf ) -> i32 {
		self.compile_tx.send( CompilerMessage::File( path.clone() ) ).unwrap();
		JACK_FRAME_WAIT
	}

	fn on_message( &mut self, msg: UiMessage ) -> i32 {
		match msg {
			UiMessage::Data( path, asm, evs ) => {
				self.path     = path;
				self.assembly = asm;
				self.events   = evs;
				self.text     = None;
				let rng = random::Generator::new(); // XXX
				let evaluator = generator::Evaluator::new( &rng );
				self.tempo = evaluator.eval( &self.assembly.tempo, ratio::Ratio::zero() );
			},
			UiMessage::Text( text ) => {
				self.text = Some( text );
			},
			UiMessage::Player( player ) => {
				self.player = player;
			},
		}

		let bgn = match self.events.get( 0 ) {
			Some( ev ) => ev.time.max( 0.0 ),
			None       => 0.0,
		};
		self.player.set_data( self.events.clone() );
		if self.autoplay && !self.player.is_playing() {
			self.player.seek( bgn ).ok();
			self.player.play().ok();
		}

		JACK_FRAME_WAIT
	}
}

impl Ui {
	fn new( compile_tx: sync::mpsc::Sender<CompilerMessage> ) -> Ui {
		Ui {
			compile_tx: compile_tx,
			path: path::PathBuf::new(),
			assembly: Assembly::default(),
			events: Vec::new(),
			tempo: 1.0,
			text: None,
			player: player::DummyPlayer::new(),
			piano_roll: pianoroll::PianoRoll::new(),
			channel: 0,
			follow: true,
			autoplay: true,
			ports: Vec::new(),
			wallpaper: None,
		}
	}

	unsafe fn draw_all( &mut self ) -> i32 {
		use imgui::*;

		let is_playing = self.player.is_playing();
		let location   = self.player.location();

		if let Some( ref text ) = self.text {
			imutil::message_dialog( "Message", text );
		}

		let mut changed = self.draw_transport();

		PushStyleColor( ImGuiCol_WindowBg as i32, 0xffffffff );
		imutil::root_begin( 0 );
			let size = GetWindowSize();
			if let Some( &(_, ref ch) ) = self.assembly.channels
				.iter().filter( |&&(i, _)| i == self.channel ).next()
			{
				let result = self.piano_roll.draw(
					&ch.score, self.assembly.len.to_float() as f32,
					(location * self.tempo) as f32,
					is_playing && self.follow, size,
				);
				if let Some( loc ) = result {
					self.player.seek( f64::max( loc as f64, 0.0 ) / self.tempo ).ok();
					changed = true;
				}
			}
			if let Some( ref wallpaper ) = self.wallpaper {
				let scale = f32::max( size.x / wallpaper.size.0 as f32, size.y / wallpaper.size.1 as f32 );
				let wsize = scale * ImVec2::new( wallpaper.size.0 as f32, wallpaper.size.1 as f32 );
				let v0 = GetWindowPos() + self.piano_roll.scroll * (size - wsize);
				(*GetWindowDrawList()).AddImage(
					wallpaper.id as _, &v0, &(v0 + wsize), &ImVec2::zero(), &ImVec2::new( 1.0, 1.0 ), 0xffff_ffff,
				);
			}
		imutil::root_end();
		PopStyleColor( 1 );

		if changed { JACK_FRAME_WAIT } else if is_playing { 1 } else { 0 }
	}

	unsafe fn draw_transport( &mut self ) -> bool {
		use imgui::*;

		let mut changed = false;

		let padding = get_style().WindowPadding;
		PushStyleVar1( ImGuiStyleVar_WindowMinSize as i32, &ImVec2::zero() );
		PushStyleVar1( ImGuiStyleVar_WindowPadding as i32, &(0.5 * padding).round() );
		SetNextWindowPos( &ImVec2::zero(), ImGuiCond_Always as i32, &ImVec2::zero() );
		Begin(
			c_str!( "Transport" ), ptr::null_mut(),
			(ImGuiWindowFlags_AlwaysAutoResize | ImGuiWindowFlags_NoMove |
			 ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoTitleBar) as i32
		);
			let size = ImVec2::new( GetFontSize() * 2.0, 0.0 );
			if Button( c_str!( "\u{f048}" ), &size ) {
				self.player.seek( 0.0 ).ok();
				changed = true;
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "\u{f04b}" ), &size ) {
				self.player.play().ok();
				changed = true;
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "\u{f04d}" ), &size ) {
				self.player.stop().ok();
				changed = true;
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "\u{f051}" ), &size ) {
				self.player.seek( self.assembly.len.to_float() / self.tempo ).ok();
				changed = true;
			}

			SameLine( 0.0, -1.0 );
			Checkbox( c_str!( "Follow" ), &mut self.follow );
			SameLine( 0.0, -1.0 );
			Checkbox( c_str!( "Autoplay" ), &mut self.autoplay );

			SameLine( 0.0, -1.0 );
			ImGui::PushItemWidth( imutil::text_size( "_Channel 00____" ).x );
			if BeginCombo( c_str!( "##Channel" ), c_str!( "Channel {:2}", self.channel ), 0 ) {
				for &(i, _) in self.assembly.channels.iter() {
					if Selectable( c_str!( "Channel {:2}", i ), i == self.channel, 0, &ImVec2::zero() ) {
						self.channel = i;
						changed = true;
					}
				}
				EndCombo();
			}
			PopItemWidth();

			SameLine( 0.0, -1.0 );
			if Button( c_str!( "Ports..." ), &ImVec2::zero() ) {
				OpenPopup( c_str!( "ports" ) );
				self.ports = self.player.ports().unwrap_or_default();
			}
			if BeginPopup( c_str!( "ports" ) ) {
				for &mut (ref port, ref mut is_conn) in self.ports.iter_mut() {
					if Checkbox( c_str!( "{}", port ), is_conn ) {
						*is_conn = if *is_conn {
							self.player.connect( port ).is_ok()
						}
						else {
							self.player.disconnect( port ).is_err()
						}
					}
				}
				EndPopup();
			}

			SameLine( 0.0, -1.0 );
			if Button( c_str!( "Generate SMF" ), &ImVec2::zero() ) {
				if let Err( e ) = self.generate_smf() {
					self.text = Some( format!( "{}", e ) );
					changed = true;
				}
			}
			SameLine( 0.0, -1.0 );
			if Button( c_str!( "\u{f021}" ), &ImVec2::zero() ) {
                self.compile_tx.send( CompilerMessage::Refresh ).unwrap();
			}
		End();
		PopStyleVar( 2 );

		changed
	}

	fn generate_smf( &self ) -> io::Result<()> {
		let smf = self.path.with_extension( "mid" );
		let mut buf = io::BufWriter::new( fs::File::create( smf )? );
		memol::smf::write_smf( &mut buf, &self.events, 480 )?;
		Ok( () )
	}
}

fn compile_task( rx: sync::mpsc::Receiver<CompilerMessage>, ui_tx: window::MessageSender<UiMessage>, bus_tx: ipc::Sender<ipc::Message> ) {
	let mut path = path::PathBuf::new();
	let mut modified = time::UNIX_EPOCH;
	loop {
		match notify::wait_file_or_channel( &path, &rx, modified ) {
			notify::WaitResult::File( v ) => {
				modified = v;
			},
			notify::WaitResult::Channel( CompilerMessage::File( v ) ) => {
				path = v;
				modified = time::UNIX_EPOCH;
				continue;
			},
			notify::WaitResult::Channel( CompilerMessage::Refresh ) => (),
			notify::WaitResult::Disconnect => break,
		}
		if path == path::PathBuf::new() {
			continue;
		}

		let msg = || -> Result<_, misc::Error> {
			let rng = random::Generator::new();
			let asm = compile( &rng, &path )?;
			let evs = assemble( &rng, &asm )?;
			bus_tx.send( &ipc::Message::Success{
				events: evs.iter().map( |e| e.clone().into() ).collect()
			} ).unwrap();
			Ok( UiMessage::Data( path.clone(), asm, evs ) )
		}().unwrap_or_else( |e| {
			UiMessage::Text( format!( "{}", e ) )
		} );
		ui_tx.send( msg );
	}
}

pub fn init_imgui( scale: f32 ) {
	let io = imgui::get_io();
	io.IniFilename = ptr::null();
	imutil::set_theme(
		imgui::ImVec4::new( 0.10, 0.10, 0.10, 1.0 ),
		imgui::ImVec4::new( 1.00, 1.00, 1.00, 1.0 ),
		imgui::ImVec4::new( 0.05, 0.05, 0.05, 1.0 ),
	);
	unsafe {
		imgui::get_style().FramePadding = imgui::ImVec2::new( 4.0, 4.0 );
		imgui::get_style().ItemSpacing  = imgui::ImVec2::new( 4.0, 4.0 );
		imgui::get_style().ScaleAllSizes( scale );

		let mut cfg = imgui::ImFontConfig::new();
		cfg.FontDataOwnedByAtlas = false;
		cfg.MergeMode     = false;
		cfg.GlyphOffset.y = (-1.0 * scale).round();
		let font = include_bytes!( "../fonts/inconsolata_regular.ttf" );
		(*io.Fonts).AddFontFromMemoryTTF(
			font.as_ptr() as *mut os::raw::c_void,
			font.len() as i32, (12.0 * scale).round(), &cfg, ptr::null(),
		);
		cfg.MergeMode     = true;
		cfg.GlyphOffset.y = 0.0;
		let font = include_bytes!( "../fonts/awesome_solid.ttf" );
		(*io.Fonts).AddFontFromMemoryTTF(
			font.as_ptr() as *mut os::raw::c_void,
			font.len() as i32, (12.0 * scale).round(), &cfg, [ 0xf000, 0xf3ff, 0 ].as_ptr(),
		);
	}
}

fn lighten_image( img: &mut image::RgbaImage, ratio: f32 ) {
	for px in img.pixels_mut() {
		let rgb = imgui::ImVec4::new( px[0] as f32, px[1] as f32, px[2] as f32, 0.0 );
		let rgb = imutil::srgb_gamma_to_linear( (1.0 / 255.0) * rgb );
		let ys = rgb.dot( &imgui::ImVec4::new( 0.2126, 0.7152, 0.0722, 0.0 ) );
		let yd = (1.0 - ratio) + ratio * ys;
		let rgb_min = f32::min( f32::min( rgb.x, rgb.y ), rgb.z );
		let rgb_max = f32::max( f32::max( rgb.x, rgb.y ), rgb.z );
		let a = 1.0;
		let a = f32::min( a, (yd - 0.0 + f32::MIN_POSITIVE) / f32::max(  f32::MIN_POSITIVE, ys - rgb_min ) );
		let a = f32::min( a, (yd - 1.0 - f32::MIN_POSITIVE) / f32::min( -f32::MIN_POSITIVE, ys - rgb_max ) );
		let rgb = a * rgb + imgui::ImVec4::constant( yd - a * ys );
		let rgb = 255.0 * imutil::srgb_linear_to_gamma( rgb ) + imgui::ImVec4::constant( 0.5 );
		px[0] = rgb.x as u8;
		px[1] = rgb.y as u8;
		px[2] = rgb.z as u8;
	}
}

fn main() {
	|| -> Result<(), Box<error::Error>> {
		#[cfg( windows )]
		unsafe {
			extern crate libloading;
			let lib = libloading::Library::new( "user32.dll" )?;
			let set_process_dpi_aware: libloading::Symbol<extern fn()> = lib.get( b"SetProcessDPIAware" )?;
			set_process_dpi_aware();
		}

		// parse command line.
		let mut opts = getopts::Options::new();
		opts.optopt  ( "s", "scale",     "Set DPI scaling.",      "VALUE"     );
		opts.optopt  ( "w", "wallpaper", "Set background image.", "FILE"      );
		opts.optmulti( "c", "connect",   "Connect to JACK port.", "PORT"      );
		opts.optopt  ( "a", "address",   "WebSocket address.",    "ADDR:PORT" );
		let args = match opts.parse( env::args().skip( 1 ) ) {
			Ok ( v ) => v,
			Err( _ ) => {
				print!( "{}", opts.usage( "Usage: memol_gui [options] [FILE]" ) );
				return Ok( () );
			},
		};
		if args.free.len() > 1 {
			return Err( getopts::Fail::UnexpectedArgument( String::new() ).into() );
		}

		// initialize IPC.
		let addr = args.opt_str( "a" ).unwrap_or( "127.0.0.1:27182".into() );
		let bus = ipc::Bus::new();
		let bus_tx: ipc::Sender<ipc::Message> = bus.create_sender();
		thread::spawn( move || {
			if let Err( err ) = bus.listen( addr, |_| () ) {
				eprintln!( "IPC: {}", err );
			}
		} );

		let (compile_tx, compile_rx) = sync::mpsc::channel();

		// initialize window.
		let scaling = args.opt_str( "s" ).map( |e| e.parse() ).unwrap_or( Ok( 2.0 ) )?;
		init_imgui( scaling );
		let mut window = window::Window::new( Ui::new( compile_tx.clone() ) )?;
		if let Some( path ) = args.opt_str( "w" ) {
			let mut wallpaper = renderer::Texture::new();
			let mut img = image::open( path )?.to_rgba();
			lighten_image( &mut img, 0.5 );
			wallpaper.upload_u32( img.as_ptr(), img.width() as i32, img.height() as i32 );
			window.ui_mut().wallpaper = Some( wallpaper );
		}

		// initialize compiler.
		let window_tx = window.create_sender();
		thread::spawn( move || compile_task( compile_rx, window_tx, bus_tx ) );
		if let Some( path ) = args.free.first() {
			compile_tx.send( CompilerMessage::File( path.into() ) ).unwrap();
		}
		else {
			window.create_sender().send( UiMessage::Text(
				"Drag and drop to open a file.".into()
			) );
		}

		// initialize player.
		let ports = args.opt_strs( "c" );
		let window_tx = window.create_sender();
		thread::spawn( move || {
			let player = match player_jack::Player::new( "memol" ) {
				Ok ( v ) => v,
				Err( v ) => {
					window_tx.send( UiMessage::Text( format!( "Error: {}", v ) ) );
					return;
				},
			};
			for port in ports {
				if let Err( v ) = player.connect( &port ) {
					window_tx.send( UiMessage::Text( format!( "Error: {}", v ) ) );
					return;
				}
			}
			window_tx.send( UiMessage::Player( player ) );
		} );

		window.event_loop()
	}().unwrap_or_else( |e| {
		println!( "Error: {}", e );
		process::exit( -1 );
	} );
}
