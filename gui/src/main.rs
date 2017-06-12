// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
#![feature( catch_expr )]
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
use std::io::prelude::*;
use imgui::ImVec2;
use memol::*;


const JACK_FRAME_WAIT: i32 = 12;


enum UiMessage {
	Data( Assembly, Vec<midi::Event> ),
	Text( String ),
}

struct Ui {
	assembly: Assembly,
	end: ratio::Ratio,
	tempo: f64, // XXX
	text: Option<String>,
	player: Box<player::Player>,
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
		unsafe { self.draw_all() }
	}

	fn on_message( &mut self, msg: UiMessage ) -> i32 {
		let n = match msg {
			UiMessage::Data( asm, evs ) => {
				self.assembly = asm;
				self.text     = None;
				let bgn = match evs.get( 0 ) {
					Some( ev ) => ev.time,
					None       => 0.0,
				};
				self.player.set_data( evs );
				self.player.seek( bgn ).unwrap_or( () );
				self.player.play().unwrap_or( () );
				JACK_FRAME_WAIT
			},
			UiMessage::Text( text ) => {
				self.assembly = Assembly::default();
				self.text     = Some( text );
				0
			},
		};

		self.end = self.assembly.channels.iter()
			.flat_map( |&(_, ref v)| v.score.notes.iter() )
			.map( |v| v.t1 )
			.max()
			.unwrap_or( ratio::Ratio::zero() );
		self.tempo = self.assembly.tempo.value( ratio::Ratio::zero() );

		n
	}
}

impl Ui {
	fn new( name: &str ) -> io::Result<Self> {
		let player = player::Player::new( name )?;
		Ok( Ui {
			assembly: Assembly::default(),
			end: ratio::Ratio::zero(),
			tempo: 1.0,
			text: None,
			player: player,
			channel: 0,
			follow: true,
			color_time_bar:  imutil::pack_color( imutil::srgb_gamma( imgui::ImVec4::new( 0.00, 0.00, 0.00, 1.00 ) ) ),
			color_time_odd:  imutil::pack_color( imutil::srgb_gamma( imgui::ImVec4::new( 0.00, 0.00, 0.00, 0.02 ) ) ),
			color_chromatic: imutil::pack_color( imutil::srgb_gamma( imgui::ImVec4::new( 0.90, 0.90, 0.90, 1.00 ) ) ),
			color_note_top:  imutil::pack_color( imutil::srgb_gamma( imgui::ImVec4::new( 0.10, 0.15, 0.20, 1.00 ) ) ),
			color_note_sub:  imutil::pack_color( imutil::srgb_gamma( imgui::ImVec4::new( 0.60, 0.70, 0.80, 1.00 ) ) ),
		} )
	}

	unsafe fn draw_all( &mut self ) -> i32 {
		use imgui::*;

		let mut count = 0;
		let is_playing = self.player.is_playing();
		let loc = (self.player.location().to_float() * self.tempo) as f32;

		if let Some( ref text ) = self.text {
			SetNextWindowPosCenter( ImGuiSetCond_Always as i32 );
			Begin( c_str!( "Message" ), ptr::null_mut(), ImGuiWindowFlags_AlwaysAutoResize as i32 );
				Text( c_str!( "{}", text ) );
			End();
		}

		SetNextWindowPos( &ImVec2::zero(), ImGuiSetCond_Once as i32 );
		Begin(
			c_str!( "Transport" ), ptr::null_mut(),
			(ImGuiWindowFlags_AlwaysAutoResize | ImGuiWindowFlags_NoMove |
			 ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoTitleBar) as i32
		);
			Button( c_str!( "Menu" ), &ImVec2::zero() );
			if BeginPopupContextItem( c_str!( "Menu" ), 0 ) {
				Checkbox( c_str!( "Follow" ), &mut self.follow );
				Checkbox( c_str!( "Repeat" ), &mut false );
				EndPopup();
			}

			SameLine( 0.0, -1.0 );
			if Button( c_str!( "<<" ), &ImVec2::zero() ) {
				self.player.seek( 0.0 ).unwrap_or( () );
				count = cmp::max( count, JACK_FRAME_WAIT );
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "Play" ), &ImVec2::zero() ) {
				self.player.play().unwrap_or( () );
				count = cmp::max( count, JACK_FRAME_WAIT );
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( "Stop" ), &ImVec2::zero() ) {
				self.player.stop().unwrap_or( () );
			}
			SameLine( 0.0, 1.0 );
			if Button( c_str!( ">>" ), &ImVec2::zero() ) {
				self.player.seek( self.end.to_float() / self.tempo ).unwrap_or( () );
				count = cmp::max( count, JACK_FRAME_WAIT );
			}

			for &(ch, _) in self.assembly.channels.iter() {
				SameLine( 0.0, -1.0 );
				RadioButton1( c_str!( "{}", ch ), &mut self.channel, ch as i32 );
			}
		End();

		imutil::begin_root( ImGuiWindowFlags_HorizontalScrollbar );
			// scrolling.
			let ctx = imutil::DrawContext::new();
			let note_size = ImVec2::new( (ctx.size().y / 8.0).ceil(), ctx.size().y / 128.0 );
			let prev = GetScrollX();
			if self.follow && is_playing {
				let next = loc * note_size.x - ctx.size().x / 4.0;
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
			count = cmp::max( count, self.draw_time_bar( &mut ctx, note_size, loc ) );
		imutil::end_root();

		cmp::max( count, if is_playing { 1 } else { 0 } )
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
			if IsItemHovered() {
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
					Text( c_str!( "     note = {}{}", sym, nnum / 12 - 1 ) );
					Text( c_str!( "gate time = {} + {}/{}",
						misc::idiv( note.t0.y, note.t0.x ),
						misc::imod( note.t0.y, note.t0.x ),
						note.t0.x,
					) );
					Text( c_str!( " duration = {}/{}", dt.y, dt.x ) );
				EndTooltip();
			}
		}
	}

	unsafe fn draw_time_bar( &mut self, ctx: &mut imutil::DrawContext, note_size: ImVec2, loc: f32 ) -> i32 {
		use imgui::*;
		let mut count = 0;

		PushStyleVar1( ImGuiStyleVar_ItemSpacing as i32, &ImVec2::zero() );
		for i in 0 .. self.end.floor() + 1 {
			SetCursorPos( &ImVec2::new( (i as f32 - 0.5) * note_size.x, 0.0 ) );
			if InvisibleButton( c_str!( "time_bar##{}", i ), &ImVec2::new( note_size.x, ctx.size().y ) ) {
				self.player.seek( i as f64 / self.tempo ).unwrap_or( () );
				count = cmp::max( count, JACK_FRAME_WAIT );
			}
		}
		PopStyleVar( 1 );

		let lt = ImVec2::new( loc * note_size.x - 1.0, 0.0          );
		let rb = ImVec2::new( loc * note_size.x - 1.0, ctx.size().y );
		ctx.add_line( lt, rb, self.color_time_bar, 1.0 );

		count
	}
}

fn compile_task( file: &str, tx: window::MessageSender<UiMessage> ) -> Result<(), Box<error::Error>> {
	loop {
		let mut buf = String::new();
		fs::File::open( file )?.read_to_string( &mut buf )?;

		let msg = do catch {
			let asm = compile( &buf )?;
			let evs = assemble( &asm )?;
			Ok( UiMessage::Data( asm, evs ) )
		}.unwrap_or_else( |e: misc::Error| {
			let (row, col) = misc::text_row_col( &buf[0 .. e.loc] );
			UiMessage::Text( format!( "error at ({}, {}): {}", row, col, e.msg ) )
		} );
		tx.send( msg )?;

		notify::notify_wait( file )?;
	}
}

fn main() {
	let f = || -> Result<(), Box<error::Error>> {
		let opts = getopts::Options::new();
		let args = opts.parse( env::args().skip( 1 ) )?;
		if args.free.len() != 1 {
			return Err( getopts::Fail::UnexpectedArgument( String::new() ).into() );
		}

		let io = imgui::get_io();
		io.IniFilename = ptr::null();
		let font = include_bytes!( "../imgui/extra_fonts/Cousine-Regular.ttf" );
		imutil::set_theme(
			imgui::ImVec4::new( 0.10, 0.10, 0.10, 1.0 ),
			imgui::ImVec4::new( 1.00, 1.00, 1.00, 1.0 ),
			imgui::ImVec4::new( 0.05, 0.05, 0.05, 1.0 ),
		);
		imutil::set_scale( 1.5, 13.0, font );

		let mut window = window::Window::new( Ui::new( "memol" )? );
		let tx = window.create_sender();
		thread::spawn( move || compile_task( &args.free[0], tx ).unwrap() );
		window.event_loop()?;

		Ok( () )
	};
	if let Err( e ) = f() {
		println!( "error: {}", e.description() );
	}
}
