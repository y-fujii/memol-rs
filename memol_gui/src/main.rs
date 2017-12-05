// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
#![windows_subsystem = "windows"]
extern crate getopts;
extern crate gl;
extern crate glutin;
extern crate image;
#[macro_use]
extern crate memol;
mod imgui;
mod renderer;
mod window;
mod imutil;
mod pianoroll;
use std::*;
use std::error::Error;
use std::io::prelude::*;
use memol::*;


const JACK_FRAME_WAIT: i32 = 12;


enum UiMessage {
	Data( Assembly, Vec<midi::Event> ),
	Text( String ),
	Player( Box<player::Player> ),
}

struct Ui {
	compile_tx: sync::mpsc::Sender<String>,
	assembly: Assembly,
	events: Vec<midi::Event>,
	tempo: f64, // XXX
	text: Option<String>,
	player: Option<Box<player::Player>>,
	piano_roll: pianoroll::PianoRoll,
	channel: i32,
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
		if let Some( path ) = path.to_str() {
			self.compile_tx.send( path.into() ).unwrap();
		}
		JACK_FRAME_WAIT
	}

	fn on_message( &mut self, msg: UiMessage ) -> i32 {
		match msg {
			UiMessage::Data( asm, evs ) => {
				self.assembly = asm;
				self.events   = evs;
				self.text     = None;
				let mut evaluator = valuegen::Evaluator::new();
				self.tempo = evaluator.eval( &self.assembly.tempo, ratio::Ratio::zero() );
			},
			UiMessage::Text( text ) => {
				self.text = Some( text );
			},
			UiMessage::Player( player ) => {
				self.player = Some( player );
			},
		}

		if let Some( ref player ) = self.player {
			let bgn = match self.events.get( 0 ) {
				Some( ev ) => ev.time.max( 0.0 ),
				None       => 0.0,
			};
			player.set_data( mem::replace( &mut self.events, Vec::new() ) );
			if self.autoplay && !player.is_playing() {
				player.seek( bgn ).unwrap_or_default();
				player.play().unwrap_or_default();
			}
		}

		JACK_FRAME_WAIT
	}
}

impl Ui {
	fn new( compile_tx: sync::mpsc::Sender<String> ) -> Ui {
		Ui {
			compile_tx: compile_tx,
			assembly: Assembly::default(),
			events: Vec::new(),
			tempo: 1.0,
			text: None,
			player: None,
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

		let is_playing;
		let location;
		match self.player {
			Some( ref player ) => {
				is_playing = player.is_playing();
				location   = player.location();
			},
			None => {
				is_playing = false;
				location   = ratio::Ratio::zero();
			},
		}

		if let Some( ref text ) = self.text {
			imutil::message_dialog( "Message", text );
		}

		let mut changed = self.draw_transport();

		PushStyleColor( ImGuiCol_WindowBg as i32, 0xffffffff );
		imutil::root_begin( 0 );
			let size = GetWindowSize();
			if let Some( &(_, ref ch) ) = self.assembly.channels.get( self.channel as usize ) {
				let result = self.piano_roll.draw(
					&ch.score, self.assembly.len.to_float() as f32,
					(location.to_float() * self.tempo) as f32,
					is_playing && self.follow, size,
				);
				if let (&Some( ref player ), Some( loc )) = (&self.player, result) {
					player.seek( f64::max( loc as f64, 0.0 ) / self.tempo ).unwrap_or_default();
					changed = true;
				}
			}
			if let Some( ref wallpaper ) = self.wallpaper {
				let scale = f32::max( size.x / wallpaper.size.0 as f32, size.y / wallpaper.size.1 as f32 );
				let wsize = ImVec2::new( wallpaper.size.0 as f32, wallpaper.size.1 as f32 ) * scale;
				let x = (size.x - wsize.x) * self.piano_roll.scroll;
				let v0 = GetWindowPos() + ImVec2::new( 0.0     + x, (size.y - wsize.y) * 0.5 );
				let v1 = GetWindowPos() + ImVec2::new( wsize.x + x, (size.y + wsize.y) * 0.5 );
				(*GetWindowDrawList()).AddImage(
					wallpaper.id as _, &v0, &v1, &ImVec2::zero(), &ImVec2::new( 1.0, 1.0 ), 0xffff_ffff,
				);
			}
		imutil::root_end();
		PopStyleColor( 1 );

		if changed { JACK_FRAME_WAIT } else if is_playing { 1 } else { 0 }
	}

	unsafe fn draw_transport( &mut self ) -> bool {
		use imgui::*;

		let player = match self.player {
			Some( ref v ) => v,
			None          => return false,
		};
		let mut changed = false;

		let padding = get_style().WindowPadding;
		PushStyleVar1( ImGuiStyleVar_WindowMinSize as i32, &ImVec2::zero() );
		PushStyleVar1( ImGuiStyleVar_WindowPadding as i32, &(padding * 0.5).round() );
		SetNextWindowPos( &ImVec2::zero(), ImGuiSetCond_Always as i32, &ImVec2::zero() );
		Begin(
			c_str!( "Transport" ), ptr::null_mut(),
			(ImGuiWindowFlags_AlwaysAutoResize | ImGuiWindowFlags_NoMove |
			 ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoTitleBar) as i32
		);
			let size = ImVec2::new( GetFontSize() * 2.0, 0.0 );
			if Button( c_str!( "\u{f048}" ), &size ) {
				player.seek( 0.0 ).unwrap_or_default();
				changed = true;
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "\u{f04b}" ), &size ) {
				player.play().unwrap_or_default();
				changed = true;
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "\u{f04d}" ), &size ) {
				player.stop().unwrap_or_default();
				changed = true;
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "\u{f051}" ), &size ) {
				player.seek( self.assembly.len.to_float() / self.tempo ).unwrap_or_default();
				changed = true;
			}

			SameLine( 0.0, -1.0 );
			Checkbox( c_str!( "Follow" ), &mut self.follow );
			SameLine( 0.0, -1.0 );
			Checkbox( c_str!( "Autoplay" ), &mut self.autoplay );

			SameLine( 0.0, -1.0 );
			if Button( c_str!( "Ports..." ), &ImVec2::zero() ) {
				OpenPopup( c_str!( "ports" ) );
				self.ports = player.ports().unwrap_or_default();
			}
			if BeginPopup( c_str!( "ports" ) ) {
				for &mut (ref port, ref mut is_conn) in self.ports.iter_mut() {
					if Checkbox( c_str!( "{}", port ), is_conn ) {
						*is_conn = if *is_conn {
							player.connect( port ).is_ok()
						}
						else {
							player.disconnect( port ).is_err()
						}
					}
				}
				EndPopup();
			}

			for (i, &(ch, _)) in self.assembly.channels.iter().enumerate() {
				SameLine( 0.0, -1.0 );
				RadioButton1( c_str!( "{}", ch ), &mut self.channel, i as i32 );
			}
		End();
		PopStyleVar( 2 );

		changed
	}
}

fn compile_task( rx: sync::mpsc::Receiver<String>, tx: window::MessageSender<UiMessage> ) {
	let mut path = String::new();
	let mut modified = time::UNIX_EPOCH;
	loop {
		match notify::wait_file_or_channel( &path, &rx, modified ) {
			notify::WaitResult::File( v ) => {
				modified = v;
			},
			notify::WaitResult::Message( v ) => {
				path = v;
				modified = time::UNIX_EPOCH;
				continue;
			},
			notify::WaitResult::Disconnect => {
				break;
			},
		}
		if path.is_empty() {
			continue;
		}

		let mut buf = String::new();
		if let Err( e ) = fs::File::open( &path ).and_then( |mut e| e.read_to_string( &mut buf ) ) {
			tx.send( UiMessage::Text( format!( "Error: {}", e.description() ) ) );
			continue;
		}

		let msg = || -> Result<_, misc::Error> {
			let asm = compile( &buf )?;
			let evs = assemble( &asm )?;
			Ok( UiMessage::Data( asm, evs ) )
		}().unwrap_or_else( |e| {
			let (row, col) = misc::text_row_col( &buf[0 .. e.loc] );
			UiMessage::Text( format!( "Compile error at ({}, {}): {}", row, col, e.msg ) )
		} );
		tx.send( msg );
	}
}

pub fn init_imgui( scale: f32 ) {
	let io = imgui::get_io();
	io.IniFilename = ptr::null();
	imutil::set_scale( scale );
	imutil::set_theme(
		imgui::ImVec4::new( 0.10, 0.10, 0.10, 1.0 ),
		imgui::ImVec4::new( 1.00, 1.00, 1.00, 1.0 ),
		imgui::ImVec4::new( 0.05, 0.05, 0.05, 1.0 ),
	);
	unsafe {
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
		let font = include_bytes!( "../fonts/awesome.otf" );
		(*io.Fonts).AddFontFromMemoryTTF(
			font.as_ptr() as *mut os::raw::c_void,
			font.len() as i32, (12.0 * scale).round(), &cfg, [ 0xf000, 0xf3ff, 0 ].as_ptr(),
		);
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

		let opts = getopts::Options::new();
		let args = opts.parse( env::args().skip( 1 ) )?;
		if args.free.len() > 1 {
			return Err( getopts::Fail::UnexpectedArgument( String::new() ).into() );
		}

		init_imgui( 2.0 );
		let (compile_tx, compile_rx) = sync::mpsc::channel();
		let mut window = window::Window::new( Ui::new( compile_tx.clone() ) )?;

		if let Ok( img ) = image::open( "wallpaper.png" ) {
			let img = img.to_rgba();
			let mut wallpaper = renderer::Texture::new();
			wallpaper.upload_u32( img.as_ptr(), img.width() as i32, img.height() as i32 );
			window.ui_mut().wallpaper = Some( wallpaper );
		}

		let window_tx = window.create_sender();
		thread::spawn( move || compile_task( compile_rx, window_tx ) );

		let window_tx = window.create_sender();
		thread::spawn( move || {
			let msg = match player::Player::new( "memol" ) {
				Ok ( v ) => UiMessage::Player( v ),
				Err( v ) => UiMessage::Text( format!( "Error: {}", v.description() ) ),
			};
			window_tx.send( msg );
		} );

		if let Some( path ) = args.free.first() {
			compile_tx.send( path.clone() )?;
		}
		else {
			window.create_sender().send( UiMessage::Text(
				"Drag and drop to open a file.".into()
			) );
		}

		window.event_loop()
	}().unwrap_or_else( |e| {
		println!( "Error: {}", e.description() );
		process::exit( -1 );
	} );
}
