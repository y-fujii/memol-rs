// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
#![windows_subsystem = "windows"]
extern crate getopts;
extern crate gl;
extern crate glutin;
#[macro_use]
extern crate memol;
mod imgui;
mod renderer;
mod window;
mod imutil;
use std::*;
use std::error::Error;
use std::io::prelude::*;
use imgui::ImVec2;
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
	end: ratio::Ratio,
	tempo: f64, // XXX
	text: Option<String>,
	player: Option<Box<player::Player>>,
	channel: i32,
	follow: bool,
	color_time_bar: u32,
	color_time_odd: u32,
	color_chromatic: u32,
	color_note_top: u32,
	color_note_sub: u32,
}

impl window::Ui<UiMessage> for Ui {
	fn on_draw( &mut self ) -> i32 {
		unsafe { self.draw_all().unwrap() }
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
				self.end = self.assembly.channels.iter()
					.flat_map( |&(_, ref v)| v.score.notes.iter() )
					.map( |v| v.t1 )
					.max()
					.unwrap_or( ratio::Ratio::zero() );
				let evaluator = valuegen::Evaluator::new();
				self.tempo = evaluator.eval( &self.assembly.tempo, ratio::Ratio::zero() );
			},
			UiMessage::Text( text ) => {
				self.assembly = Assembly::default();
				self.events   = Vec::new();
				self.text     = Some( text );
				self.end      = ratio::Ratio::zero();
				self.tempo    = 1.0;
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
			player.seek( bgn ).unwrap_or( () );
			player.play().unwrap_or( () );
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
			end: ratio::Ratio::zero(),
			tempo: 1.0,
			text: Some( "Drag and drop to open a file.".into() ),
			player: None,
			channel: 0,
			follow: true,
			color_time_bar:  imutil::pack_color( imutil::srgb_gamma( imgui::ImVec4::new( 0.00, 0.00, 0.00, 1.00 ) ) ),
			color_time_odd:  imutil::pack_color( imutil::srgb_gamma( imgui::ImVec4::new( 0.00, 0.00, 0.00, 0.02 ) ) ),
			color_chromatic: imutil::pack_color( imutil::srgb_gamma( imgui::ImVec4::new( 0.90, 0.90, 0.90, 1.00 ) ) ),
			color_note_top:  imutil::pack_color( imutil::srgb_gamma( imgui::ImVec4::new( 0.10, 0.15, 0.20, 1.00 ) ) ),
			color_note_sub:  imutil::pack_color( imutil::srgb_gamma( imgui::ImVec4::new( 0.60, 0.70, 0.80, 1.00 ) ) ),
		}
	}

	unsafe fn draw_all( &mut self ) -> Result<i32, Box<error::Error>> {
		use imgui::*;

		if let Some( ref text ) = self.text {
			imutil::message_dialog( "Message", text );
		}

		let player = match self.player {
			Some( ref v ) => v,
			None          => return Ok( 0 ),
		};
		let is_playing = player.is_playing();
		let location   = (player.location().to_float() * self.tempo) as f32;

		let mut count = 0;

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
				player.seek( 0.0 )?;
				count = cmp::max( count, JACK_FRAME_WAIT );
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "\u{f04b}" ), &size ) {
				player.play()?;
				count = cmp::max( count, JACK_FRAME_WAIT );
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "\u{f04d}" ), &size ) {
				player.stop()?;
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "\u{f051}" ), &size ) {
				player.seek( self.end.to_float() / self.tempo )?;
				count = cmp::max( count, JACK_FRAME_WAIT );
			}

			SameLine( 0.0, -1.0 );
			Checkbox( c_str!( "Follow" ), &mut self.follow );

			for &(ch, _) in self.assembly.channels.iter() {
				SameLine( 0.0, -1.0 );
				RadioButton1( c_str!( "{}", ch ), &mut self.channel, ch as i32 );
			}
		End();
		PopStyleVar( 2 );

		PushStyleColor( ImGuiCol_WindowBg as i32, 0xffffffff );
		imutil::begin_root( ImGuiWindowFlags_HorizontalScrollbar );
			// scrolling.
			let ctx = imutil::DrawContext::new();
			let note_size = ImVec2::new( (ctx.size().y / 8.0).ceil(), ctx.size().y / 128.0 );
			let prev = GetScrollX();
			if self.follow && is_playing {
				let next = location * note_size.x - ctx.size().x / 4.0;
				SetScrollX( prev * 0.9375 + next * 0.0625 );
			}
			else {
				let delta = GetMouseDragDelta( 0, -1.0 );
				SetScrollX( prev + delta.x * 0.25 );
				if delta.x != 0.0 {
					count = cmp::max( count, 1 );
				}
			}

			// rendering.
			let mut ctx = imutil::DrawContext::new();
			self.draw_background( &mut ctx, note_size );
			for &(ch, ref ir) in self.assembly.channels.iter() {
				if ch != self.channel as usize {
					self.draw_notes( &mut ctx, &ir.score, note_size, self.color_note_sub );
				}
			}
			for &(ch, ref ir) in self.assembly.channels.iter() {
				if ch == self.channel as usize {
					self.draw_notes( &mut ctx, &ir.score, note_size, self.color_note_top );
				}
			}

			if let Some( seek ) = self.draw_time_bar( &mut ctx, note_size, location ) {
				player.seek( seek )?;
				count = cmp::max( count, JACK_FRAME_WAIT );
			}
		imutil::end_root();
		PopStyleColor( 1 );

		count = cmp::max( count, if is_playing { 1 } else { 0 } );
		Ok( count )
	}

	unsafe fn draw_background( &self, ctx: &mut imutil::DrawContext, note_size: ImVec2 ) {
		use imgui::*;

		let end = self.end.to_float() as f32;
		for i in 0 .. (128 + 11) / 12 {
			for j in [ 1, 3, 6, 8, 10 ].iter() {
				let lt = ImVec2::new( 0.0,               (127 - i * 12 - j) as f32 * note_size.y );
				let rb = ImVec2::new( end * note_size.x, (128 - i * 12 - j) as f32 * note_size.y );
				ctx.add_rect_filled( lt, rb, self.color_chromatic, 1.0, !0 );
			}
		}

		let mut i = 1;
		while i <= self.end.floor() {
			let lt = ImVec2::new( (i + 0) as f32 * note_size.x, 0.0          );
			let rb = ImVec2::new( (i + 1) as f32 * note_size.x, ctx.size().y );
			ctx.add_rect_filled( lt, rb, self.color_time_odd, 1.0, !0 );
			i += 2;
		}
	}

	unsafe fn draw_notes( &self, ctx: &mut imutil::DrawContext, ir: &scoregen::Ir, note_size: ImVec2, color: u32 ) {
		use imgui::*;

		for note in ir.notes.iter() {
			let nnum = match note.nnum {
				Some( v ) => v,
				None      => continue,
			};

			let x0 = ImVec2::new( note.t0.to_float() as f32 * note_size.x,       (127 - nnum) as f32 * note_size.y );
			let x1 = ImVec2::new( note.t1.to_float() as f32 * note_size.x - 1.0, (128 - nnum) as f32 * note_size.y );
			ctx.add_rect_filled( x0, x1, color, note_size.y * 0.25, !0 );

			let dt = note.t1 - note.t0;
			SetCursorPos( &x0 );
			Dummy( &ImVec2::new( dt.to_float() as f32 * note_size.x - 1.0, note_size.y ) );
			if IsItemHovered( ImGuiHoveredFlags_Default as i32 ) {
				BeginTooltip();
					let sym = match nnum % 12 {
						 0 => "C",  1 => "C+",
						 2 => "D",  3 => "D+",
						 4 => "E",
						 5 => "F",  6 => "F+",
						 7 => "G",  8 => "G+",
						 9 => "A", 10 => "A+",
						11 => "B",
						 _ => panic!(),
					};
					imutil::show_text( &format!( "     note = {}{}", sym, nnum / 12 - 1 ) );
					imutil::show_text( &format!( "gate time = {} + {}/{}",
						misc::idiv( note.t0.y, note.t0.x ),
						misc::imod( note.t0.y, note.t0.x ),
						note.t0.x,
					) );
					imutil::show_text( &format!( " duration = {}/{}", dt.y, dt.x ) );
				EndTooltip();
			}
		}
	}

	unsafe fn draw_time_bar( &self, ctx: &mut imutil::DrawContext, note_size: ImVec2, loc: f32 ) -> Option<f64> {
		use imgui::*;
		let mut seek = None;

		PushStyleVar1( ImGuiStyleVar_ItemSpacing as i32, &ImVec2::zero() );
		for i in 0 .. self.end.floor() + 1 {
			SetCursorPos( &ImVec2::new( (i as f32 - 0.5) * note_size.x, 0.0 ) );
			if InvisibleButton( c_str!( "time_bar##{}", i ), &ImVec2::new( note_size.x, ctx.size().y ) ) {
				seek = Some( i as f64 / self.tempo );
			}
		}
		PopStyleVar( 1 );

		let lt = ImVec2::new( loc * note_size.x - 1.0, 0.0          );
		let rb = ImVec2::new( loc * note_size.x - 1.0, ctx.size().y );
		ctx.add_line( lt, rb, self.color_time_bar, 1.0 );

		seek
	}
}

fn compile_task( rx: sync::mpsc::Receiver<String>, tx: window::MessageSender<UiMessage> ) {
	let mut path = String::new();
	let mut modified = time::UNIX_EPOCH;
	loop {
		match rx.recv_timeout( time::Duration::from_millis( 100 ) ) {
			Ok( v ) => {
				path = v;
				modified = time::UNIX_EPOCH;
			},
			Err( sync::mpsc::RecvTimeoutError::Timeout )      => (),
			Err( sync::mpsc::RecvTimeoutError::Disconnected ) => return,
		}
		if path.is_empty() {
			continue;
		}
		|| -> Result<_, Box<error::Error>> {
			if mem::replace( &mut modified, fs::metadata( &path )?.modified()? ) == modified {
				return Ok( () );
			}

			let mut buf = String::new();
			fs::File::open( &path )?.read_to_string( &mut buf )?;

			let msg = || -> Result<_, misc::Error> {
				let asm = compile( &buf )?;
				let evs = assemble( &asm )?;
				Ok( UiMessage::Data( asm, evs ) )
			}().unwrap_or_else( |e| {
				let (row, col) = misc::text_row_col( &buf[0 .. e.loc] );
				UiMessage::Text( format!( "Compile error at ({}, {}): {}", row, col, e.msg ) )
			} );
			tx.send( msg );

			Ok( () )
		}().unwrap_or_else( |e|
			tx.send( UiMessage::Text( format!( "Error: {}", e.description() ) ) )
		);
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

		window.event_loop()
	}().unwrap_or_else( |e| {
		println!( "Error: {}", e.description() );
		process::exit( -1 );
	} );
}
