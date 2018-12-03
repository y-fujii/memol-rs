// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
#![windows_subsystem = "windows"]
extern crate getopts;
extern crate gl;
extern crate glutin;
extern crate image;
extern crate clipboard;
extern crate memol;
extern crate memol_cli;
#[macro_use]
mod imutil;
mod imgui;
mod renderer;
mod window;
mod compile_thread;
mod model;
mod piano_roll;
mod main_widget;
use std::*;
use memol::*;
use memol_cli::{ ipc, player, player_jack };
use memol_cli::player::Player;


const JACK_FRAME_WAIT: i32 = 12;

enum UiMessage {
	Data( path::PathBuf, Assembly, Vec<midi::Event> ),
	Text( String ),
	Player( Box<player::Player> ),
}

fn init_imgui( scale: f32 ) {
	let scale = f32::sqrt( scale );
	let io = imgui::get_io();
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
		cfg.GlyphOffset.y = 0.0;
		let font = include_bytes!( "../fonts/inconsolata_regular.ttf" );
		(*io.Fonts).AddFontFromMemoryTTF(
			font.as_ptr() as *mut os::raw::c_void,
			font.len() as i32, (14.0 * scale).round(), &cfg, ptr::null(),
		);
		cfg.MergeMode     = true;
		cfg.GlyphOffset.y = (0.5 * scale).round();
		let font = include_bytes!( "../fonts/awesome_solid.ttf" );
		(*io.Fonts).AddFontFromMemoryTTF(
			font.as_ptr() as *mut os::raw::c_void,
			font.len() as i32, (14.0 * scale).round(), &cfg, [ 0xf000, 0xf3ff, 0 ].as_ptr(),
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
		// parse command line.
		let mut opts = getopts::Options::new();
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
		let addr = args.opt_str( "a" ).unwrap_or( "127.0.0.1:27182".into() );
		let ports = args.opt_strs( "c" );
		let wallpaper = args.opt_str( "w" );

		// create instances.
		let mut compiler = compile_thread::CompileThread::new();
		let bus = ipc::Bus::new();
		let model = cell::RefCell::new( model::Model::new( compiler.create_sender(), bus.create_sender() ) );
		let mut widget = main_widget::MainWidget::new();
		let mut window = window::Window::new()?;

		// initialize window.
		init_imgui( window.hidpi_factor() as f32 );
		window.update_font();
		if let Some( path ) = wallpaper {
			let mut img = image::open( path )?.to_rgba();
			lighten_image( &mut img, 0.5 );
			let mut wallpaper = renderer::Texture::new();
			wallpaper.upload_u32( img.as_ptr(), img.width() as i32, img.height() as i32 );
			widget.wallpaper = Some( wallpaper );
		}
		else {
			window.set_background( 1.0, 1.0, 1.0, 1.0 );
		}

		window.on_draw( || {
			let changed = widget.draw( &mut model.borrow_mut() );
			if changed { JACK_FRAME_WAIT } else { 0 }
		} );
		window.on_message( |msg| {
			match msg {
				UiMessage::Data( path, asm, evs ) => {
					model.borrow_mut().set_data( path, asm, evs );
				},
				UiMessage::Text( text ) => {
					model.borrow_mut().text = Some( text );
				},
				UiMessage::Player( player ) => {
					model.borrow_mut().player = player;
				},
			}
			JACK_FRAME_WAIT
		} );
		window.on_file_dropped( |path| {
			model.borrow_mut().compile_tx.send( compile_thread::Message::File( path.clone() ) ).unwrap();
			0
		} );

		// initialize compiler.
		compiler.on_success( {
			let bus_tx = bus.create_sender();
			let window_tx = window.create_sender();
			move |path, asm, evs| {
				bus_tx.send( &ipc::Message::Success{
					events: evs.iter().map( |e| e.clone().into() ).collect()
				} ).unwrap();
				window_tx.send( UiMessage::Data( path.clone(), asm, evs ) );
			}
		} );
		compiler.on_failure( {
			let window_tx = window.create_sender();
			move |text| {
				window_tx.send( UiMessage::Text( text ) );
			}
		} );
		if let Some( path ) = args.free.first() {
			compiler.create_sender().send( compile_thread::Message::File( path.into() ) ).unwrap();
		}
		else {
			window.create_sender().send( UiMessage::Text(
				"Drag and drop to open a file.".into()
			) );
		}

		// spawn threads.
		compiler.spawn();
		thread::spawn( move || {
			if let Err( err ) = bus.listen( addr, |_| () ) {
				eprintln!( "IPC: {}", err );
			}
		} );
		thread::spawn( {
			let window_tx = window.create_sender();
			move || {
				let player = match player_jack::Player::new( "memol" ) {
					Ok ( v ) => v,
					Err( v ) => {
						window_tx.send( UiMessage::Text( format!( "Error: {}", v ) ) );
						return;
					},
				};
				for port in ports {
					if let Err( v ) = player.connect_to( &port ) {
						window_tx.send( UiMessage::Text( format!( "Error: {}", v ) ) );
					}
				}
				window_tx.send( UiMessage::Player( player ) );
			}
		} );

		window.event_loop()
	}().unwrap_or_else( |e| {
		eprintln!( "Error: {}", e );
		process::exit( -1 );
	} );
}
