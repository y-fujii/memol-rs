// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
#![windows_subsystem = "windows"]
#[macro_use]
mod imutil;
mod imgui;
mod renderer;
mod window;
mod compile_thread;
mod model;
mod sequencer_widget;
mod main_widget;
use std::*;
use memol::*;
use memol_cli::{ player, player_net, player_jack };
use memol_cli::player::PlayerExt;


const JACK_FRAME_WAIT: i32 = 12;

enum UiMessage {
	Data( path::PathBuf, String, Assembly, Vec<midi::Event> ),
	Text( String ),
	Midi( Vec<midi::Event> ),
}

fn init_imgui( scale: f32 ) {
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
			font.len() as i32, (14.0 * scale).round(), &cfg, [ 0x20, 0xff, 0x2026, 0x2027, 0 ].as_ptr(),
		);
		cfg.MergeMode     = true;
		cfg.GlyphOffset.y = scale.round();
		let font = include_bytes!( "../fonts/awesome_solid.ttf" );
		(*io.Fonts).AddFontFromMemoryTTF(
			font.as_ptr() as *mut os::raw::c_void,
			font.len() as i32, (14.0 * scale).round(), &cfg, [ 0xf000, 0xf7ff, 0 ].as_ptr(),
		);
	}
}

fn lighten_image( img: &mut image::RgbaImage, ratio: f32 ) {
	for px in img.pixels_mut() {
		let rgb = imgui::ImVec4::new( px[0] as f32, px[1] as f32, px[2] as f32, 0.0 );
		let rgb = imutil::srgb_gamma_to_linear( (1.0 / 255.0) * rgb );
		/*
		let ys = rgb.dot( &imgui::ImVec4::new( 0.2126, 0.7152, 0.0722, 0.0 ) );
		let yd = (1.0 - ratio) + ratio * ys;
		let rgb_min = f32::min( f32::min( rgb.x, rgb.y ), rgb.z );
		let rgb_max = f32::max( f32::max( rgb.x, rgb.y ), rgb.z );
		let a = 1.0;
		let a = f32::min( a, (yd - 0.0 + f32::MIN_POSITIVE) / f32::max(  f32::MIN_POSITIVE, ys - rgb_min ) );
		let a = f32::min( a, (yd - 1.0 - f32::MIN_POSITIVE) / f32::min( -f32::MIN_POSITIVE, ys - rgb_max ) );
		let rgb = a * rgb + imgui::ImVec4::constant( yd - a * ys );
		*/
		let rgb = imgui::ImVec4::constant( 1.0 - ratio ) + ratio * rgb;
		let rgb = 255.0 * imutil::srgb_linear_to_gamma( rgb ) + imgui::ImVec4::constant( 0.5 );
		px[0] = rgb.x as u8;
		px[1] = rgb.y as u8;
		px[2] = rgb.z as u8;
	}
}

fn main() {
	|| -> Result<(), Box<dyn error::Error>> {
		// parse the command line.
		let mut opts = getopts::Options::new();
		opts.optopt  ( "w", "wallpaper", "Set an background image.", "FILE" );
		opts.optflag ( "j", "jack",      "Use JACK (Default on Linux)." );
		opts.optmulti( "c", "connect",   "Connect to specified JACK ports.", "PORT" );
		opts.optflag ( "n", "vst",       "Use VST plugins (Default on non-Linux OS)." );
		opts.optflag ( "a", "any",       "Allow connection from remote VSTs." );
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

		// create instances.
		let mut compiler = compile_thread::CompileThread::new();
		let model = cell::RefCell::new( model::Model::new( compiler.create_sender() ) );
		let mut widget = main_widget::MainWidget::new();
		let mut window = window::Window::new()?;

		// initialize a window.
		init_imgui( window.hidpi_factor() as f32 );
		window.update_font();
		if let Some( Ok( img ) ) = args.opt_str( "w" ).map( image::open ) {
			let mut img = img.to_rgba();
			lighten_image( &mut img, 0.5 );
			let mut wallpaper = renderer::Texture::new();
			wallpaper.upload_u32( img.as_ptr(), img.width() as i32, img.height() as i32 );
			widget.wallpaper = Some( wallpaper );
		}
		else {
			window.set_background( imgui::ImVec4::constant( 1.0 ) );
		}

		window.on_draw( || {
			let changed = unsafe { widget.draw( &mut model.borrow_mut() ) };
			if changed { JACK_FRAME_WAIT } else { 0 }
		} );
		window.on_message( |msg| {
			match msg {
				UiMessage::Data( path, code, asm, evs ) => {
					model.borrow_mut().set_data( path, code, asm, evs );
				},
				UiMessage::Text( text ) => {
					model.borrow_mut().text = Some( text );
				},
				UiMessage::Midi( evs ) => {
					model.borrow_mut().handle_midi_inputs( &evs );
				},
			}
			JACK_FRAME_WAIT
		} );
		window.on_file_dropped( |path| {
			model.borrow_mut().compile_tx.send( compile_thread::Message::File( path.clone() ) ).unwrap();
			0
		} );

		// initialize a player.
		let addr = (if args.opt_present( "a" ) { net::Ipv6Addr::UNSPECIFIED } else { net::Ipv6Addr::LOCALHOST }, 27182);
		let mut player: Box<dyn player::Player> = match (args.opt_present( "j" ),  args.opt_present( "n" )) {
			(true, false) => Box::new( player_jack::Player::new( "memol" )? ),
			(false, true) => Box::new( player_net::Player::new( addr )? ),
			_ => {
				#[cfg( all( target_family = "unix", not( target_os = "macos" ) ) )]
				let player = player_jack::Player::new( "memol" );
				#[cfg( not( all( target_family = "unix", not( target_os = "macos" ) ) ) )]
				let player = player_net::Player::new( addr );
				Box::new( player? )
			},
		};
		player.on_received( {
			let window_tx = window.create_sender();
			move |evs| window_tx.send( UiMessage::Midi( evs.to_vec() ) )
		} );
		for port in args.opt_strs( "c" ) {
			player.connect_to( &port )?;
		}
		model.borrow_mut().player = player;

		// initialize a compiler.
		compiler.on_success( {
			let window_tx = window.create_sender();
			move |path, asm, evs| {
				let code = fs::read_to_string( &path )
					.unwrap_or_else( |_| String::new() );
				window_tx.send( UiMessage::Data( path, code, asm, evs ) );
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
		compiler.spawn();

		// start an event loop.
		window.event_loop()
	}().unwrap_or_else( |e| {
		eprintln!( "Error: {}", e );
		process::exit( -1 );
	} );
}
