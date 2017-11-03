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
	dragging: bool,
	time_scale: f32,
	color_line_0: u32,
	color_line_1: u32,
	color_note: u32,
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
				let mut evaluator = valuegen::Evaluator::new();
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
			dragging: false,
			time_scale: 24.0,
			color_line_0: imutil::pack_color( imutil::srgb_gamma( imgui::ImVec4::new( 0.40, 0.40, 0.40, 1.00 ) ) ),
			color_line_1: imutil::pack_color( imutil::srgb_gamma( imgui::ImVec4::new( 0.80, 0.80, 0.80, 1.00 ) ) ),
			color_note:   imutil::pack_color( imutil::srgb_gamma( imgui::ImVec4::new( 0.15, 0.15, 0.15, 1.00 ) ) ),
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
		let content_h = imutil::root_size().y - get_style().ScrollbarSize;
		let unit = content_h / 128.0;
		let content_w = unit * self.time_scale * (self.end + 1).to_float() as f32;
		SetNextWindowContentSize( &ImVec2::new( content_w, content_h ) );
		imutil::root_begin( ImGuiWindowFlags_HorizontalScrollbar );
			/* mouse operation. */ {
				let dx = GetMouseDragDelta( 0, -1.0 ).x;
				self.dragging |= dx != 0.0;

				if self.dragging {
					SetScrollX( GetScrollX() + dx * 0.25 );
					count = cmp::max( count, 1 );
				}
				else {
					SetCursorScreenPos( &GetWindowPos() );
					if InvisibleButton( c_str!( "background" ), &GetWindowSize() ) {
						let x = (GetMousePos().x - imutil::window_origin().x) / (unit * self.time_scale) - 0.5;
						player.seek( f64::max( x as f64, 0.0 ) / self.tempo )?;
						count = cmp::max( count, JACK_FRAME_WAIT );
					}
					else if self.follow && is_playing {
						let next = (location + 0.5) * self.time_scale * unit - (1.0 / 6.0) * GetWindowSize().x;
						SetScrollX( (31.0 / 32.0) * GetScrollX() + (1.0 / 32.0) * next );
						count = cmp::max( count, 1 );
					}
				}

				self.dragging &= !IsMouseReleased( 0 );
			}

			/* rendering. */ {
				let mut ctx = imutil::DrawContext::new( unit, ImVec2::new( unit * self.time_scale * 0.5, 0.0 ) );
				self.draw_background( &mut ctx );
				for &(ch, ref ir) in self.assembly.channels.iter() {
					if ch == self.channel as usize {
						self.draw_notes( &mut ctx, &ir.score, self.color_note );
					}
				}
				self.draw_time_bar( &mut ctx, location );
			}
		imutil::root_end();
		PopStyleColor( 1 );

		count = cmp::max( count, if is_playing { 1 } else { 0 } );
		Ok( count )
	}

	unsafe fn draw_background( &self, ctx: &mut imutil::DrawContext ) {
		use imgui::*;

		let end = self.end.to_float() as f32;

		for i in 0 .. self.end.floor() + 1 {
			let ys = [
				(43.5 - 24.0, 57.5 - 24.0),
				(43.5       , 57.5       ),
				(64.5       , 77.5       ),
				(64.5 + 24.0, 77.5 + 24.0),
			];
			for &(y0, y1) in ys.iter() {
				let v0 = ImVec2::new( self.time_scale * i as f32, y0 );
				let v1 = ImVec2::new( self.time_scale * i as f32, y1 );
				ctx.add_line( v0, v1, self.color_line_0, 0.25 );
			}
		}

		let ys = [
			43,      47,      50,      53,      57,
			43 - 24, 47 - 24, 50 - 24, 53 - 24, 57 - 24,
			64,      67,      71,      74,      77,
			64 + 24, 67 + 24, 71 + 24, 74 + 24, 77 + 24,
		];
		for &i in ys.iter() {
			let v0 = ImVec2::new( self.time_scale * 0.0 - 0.5 * 0.25, i as f32 + 0.5 );
			let v1 = ImVec2::new( self.time_scale * end + 0.5 * 0.25, i as f32 + 0.5 );
			ctx.add_line( v0, v1, self.color_line_0, 0.25 );
		}

		let ys = [
			36, 40,
			60, 81, 84,
		];
		for &i in ys.iter() {
			let v0 = ImVec2::new( 0.0                   - 0.5 * 0.25, i as f32 + 0.5 );
			let v1 = ImVec2::new( end * self.time_scale + 0.5 * 0.25, i as f32 + 0.5 );
			ctx.add_line( v0, v1, self.color_line_1, 0.25 );
		}
	}

	unsafe fn draw_notes( &self, ctx: &mut imutil::DrawContext, ir: &scoregen::Ir, color: u32 ) {
		use imgui::*;

		for note in ir.notes.iter() {
			let nnum = match note.nnum {
				Some( v ) => v,
				None      => continue,
			};

			let x0 = ImVec2::new( self.time_scale * note.t0.to_float() as f32, nnum as f32 + 0.0 );
			let x1 = ImVec2::new( self.time_scale * note.t1.to_float() as f32, nnum as f32 + 1.0 );
			ctx.add_rect_filled( x0, x1, color, 0.5, !0 );

			let (lt, rb) = ctx.transform_rect( x0, x1 );
			SetCursorScreenPos( &lt );
			Dummy( &(rb - lt) );
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
					let dt = note.t1 - note.t0;
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

	unsafe fn draw_time_bar( &self, ctx: &mut imutil::DrawContext, loc: f32 ) {
		let v0 = ImVec2::new( self.time_scale * loc,   0.0 );
		let v1 = ImVec2::new( self.time_scale * loc, 128.0 );
		ctx.add_line( v0, v1, self.color_line_0, 0.25 );
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
