// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
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

		let player = match self.player {
			Some( ref v ) => v,
			None          => return false,
		};
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

		let msg = || -> Result<_, misc::Error> {
			let asm = compile( &path::PathBuf::from( &path ) )?;
			let evs = assemble( &asm )?;
			Ok( UiMessage::Data( asm, evs ) )
		}().unwrap_or_else( |e| {
			UiMessage::Text( e.message() )
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

		let mut opts = getopts::Options::new();
		opts.optopt( "s", "", "", "" );
		opts.optopt( "b", "", "", "" );
		opts.optmulti( "c", "", "", "" );
		let args = match opts.parse( env::args().skip( 1 ) ) {
			Ok ( v ) => v,
			Err( _ ) => {
				println!( "Usage: memol_gui (-s SCALING_FACTOR)? (-b BACKGROUND_IMAGE)? (-c JACK_PORT)* (FILE)?" );
				return Ok( () );
			},
		};
		if args.free.len() > 1 {
			return Err( getopts::Fail::UnexpectedArgument( String::new() ).into() );
		}

		let scaling = args.opt_str( "s" ).map( |e| e.parse() ).unwrap_or( Ok( 2.0 ) )?;
		init_imgui( scaling );
		let (compile_tx, compile_rx) = sync::mpsc::channel();
		let mut window = window::Window::new( Ui::new( compile_tx.clone() ) )?;

		let window_tx = window.create_sender();
		thread::spawn( move || compile_task( compile_rx, window_tx ) );

		let ports = args.opt_strs( "c" );
		let window_tx = window.create_sender();
		thread::spawn( move || {
			let player = match player::Player::new( "memol" ) {
				Ok ( v ) => v,
				Err( v ) => {
					window_tx.send( UiMessage::Text( format!( "Error: {}", v.description() ) ) );
					return;
				},
			};
			for port in ports {
				if let Err( v ) = player.connect( &port ) {
					window_tx.send( UiMessage::Text( format!( "Error: {}", v.description() ) ) );
					return;
				}
			}
			window_tx.send( UiMessage::Player( player ) );
		} );

		if let Some( path ) = args.free.first() {
			compile_tx.send( path.clone() )?;
		}
		else {
			window.create_sender().send( UiMessage::Text(
				"Drag and drop to open a file.".into()
			) );
		}

		if let Some( path ) = args.opt_str( "b" ) {
			let mut wallpaper = renderer::Texture::new();
			let mut img = image::open( path )?.to_rgba();
			lighten_image( &mut img, 0.5 );
			wallpaper.upload_u32( img.as_ptr(), img.width() as i32, img.height() as i32 );
			window.ui_mut().wallpaper = Some( wallpaper );
		}

		window.event_loop()
	}().unwrap_or_else( |e| {
		println!( "Error: {}", e.description() );
		process::exit( -1 );
	} );
}
