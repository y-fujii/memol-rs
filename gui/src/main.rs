// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
#![feature( untagged_unions )]
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
	Data( Vec<Vec<irgen::FlatNote>>, Vec<midi::Event> ),
	Text( String ),
}

struct Ui {
	data: Vec<Vec<irgen::FlatNote>>,
	text: Option<String>,
	loc_end: ratio::Ratio,
	player: Box<player::Player>,
	channel: i32,
	follow: bool,
	color_line: u32,
	color_time_bar: u32,
	color_chromatic: u32,
	color_note_top: u32,
	color_note_sub: u32,
}

impl window::Ui<UiMessage> for Ui {
	fn on_draw( &mut self ) -> i32 {
		unsafe { self.draw_all() }
	}

	fn on_message( &mut self, msg: UiMessage ) {
		match msg {
			UiMessage::Data( irs, evs ) => {
				self.data = irs;
				self.player.set_data( evs );
				self.text = None;
			},
			UiMessage::Text( text ) => {
				self.data = Vec::new();
				self.player.set_data( Vec::new() );
				self.text = Some( text );
			},
		}

		self.loc_end = self.data.iter()
			.flat_map( |v| v.iter() )
			.map( |v| v.end )
			.max()
			.unwrap_or( ratio::Ratio::new( 0, 1 ) );
	}
}

impl Ui {
	fn new( name: &str ) -> io::Result<Self> {
		let player = player::Player::new( name )?;
		Ok( Ui {
			data: Vec::new(),
			text: None,
			loc_end: ratio::Ratio::new( 0, 1 ),
			player: player,
			channel: 0,
			follow: false,
			color_line:      imutil::srgb_gamma( 0.5, 0.5, 0.5, 1.0 ),
			color_time_bar:  imutil::srgb_gamma( 0.0, 0.0, 0.0, 1.0 ),
			color_chromatic: imutil::srgb_gamma( 0.9, 0.9, 0.9, 1.0 ),
			color_note_top:  imutil::srgb_gamma( 0.1, 0.3, 0.4, 1.0 ),
			color_note_sub:  imutil::srgb_gamma( 0.7, 0.9, 1.0, 1.0 ),
		} )
	}

	unsafe fn draw_all( &mut self ) -> i32 {
		use imgui::*;

		let mut count = 0;
		let is_playing = self.player.is_playing();
		let loc = (self.player.location() / 2).to_float() as f32;

		if let Some( ref text ) = self.text {
			Begin( c_str!( "Message" ), &mut true, WindowFlags_AlwaysAutoResize );
				Text( c_str!( "{}", text ) );
			End();
		}

		SetNextWindowPos( &ImVec2::zero(), SetCond_Once );
		Begin(
			c_str!( "Transport" ), &mut true,
			WindowFlags_AlwaysAutoResize | WindowFlags_NoMove |
			WindowFlags_NoResize | WindowFlags_NoTitleBar
		);
			Button( c_str!( "Menu" ), &ImVec2::zero() );
			if BeginPopupContextItem( c_str!( "Menu" ), 0 ) {
				Checkbox( c_str!( "Follow" ), &mut self.follow );
				Checkbox( c_str!( "Repeat" ), &mut false );
				Checkbox( c_str!( "Play on Save" ), &mut true );
				EndPopup();
			}

			SameLine( 0.0, -1.0 );
			if Button( c_str!( "<<" ), &ImVec2::zero() ) {
				self.player.seek( ratio::Ratio::new( 0, 1 ) ).unwrap_or( () );
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
				self.player.seek( self.loc_end * 2 ).unwrap_or( () );
				count = cmp::max( count, JACK_FRAME_WAIT );
			}

			SameLine( 0.0, -1.0 );
			for i in 0 .. self.data.len() as i32 {
				RadioButton1( c_str!( "##{}", i ), &mut self.channel, i );
				SameLine( 0.0, 1.0 );
			}
		End();

		imutil::begin_root( WindowFlags_HorizontalScrollbar );
			let ctx = imutil::DrawContext::new();
			let note_size = ImVec2::new( (ctx.size.y / 8.0).ceil(), ctx.size.y / 128.0 );
			if self.follow && is_playing {
				SetScrollX( loc * note_size.x - ctx.size.x / 2.0 );
			}
			else {
				count = cmp::max( count, self.drag_scroll() );
			}

			let mut ctx = imutil::DrawContext::new();
			self.draw_background( &mut ctx, note_size );
			for (i, ir) in self.data.iter().enumerate() {
				if i != self.channel as usize {
					self.draw_notes( &mut ctx, ir, note_size, self.color_note_sub );
				}
			}
			if (self.channel as usize) < self.data.len() {
				self.draw_notes( &mut ctx, &self.data[self.channel as usize], note_size, self.color_note_top );
			}
			count = cmp::max( count, self.draw_time_bar( &mut ctx, note_size, loc ) );
		imutil::end_root();

		cmp::max( count, if is_playing { 1 } else { 0 } )
	}

	unsafe fn drag_scroll( &self ) -> i32 {
		use imgui::*;
		let delta = GetMouseDragDelta( 1, -1.0 );
		SetScrollX( GetScrollX() + 0.25 * delta.x );
		if delta.x != 0.0 { 1 } else { 0 }
	}

	unsafe fn draw_background( &self, ctx: &mut imutil::DrawContext, note_size: ImVec2 ) {
		use imgui::*;

		let loc_end = self.loc_end.to_float() as f32;
		for i in 0 .. (128 + 11) / 12 {
			let lt = ImVec2::new( 0.0,                   (128 - i * 12) as f32 * note_size.y );
			let rb = ImVec2::new( loc_end * note_size.x, (128 - i * 12) as f32 * note_size.y );
			ctx.add_line( lt, rb, self.color_line, 1.0 );
			for j in [ 1, 3, 6, 8, 10 ].iter() {
				let lt = ImVec2::new( 0.0,                   (127 - i * 12 - j) as f32 * note_size.y );
				let rb = ImVec2::new( loc_end * note_size.x, (128 - i * 12 - j) as f32 * note_size.y );
				ctx.add_rect_filled( lt, rb, self.color_chromatic, 0.0, !0 );
			}
		}

		for i in 0 .. self.loc_end.to_int() + 1 {
			let lt = ImVec2::new( i as f32 * note_size.x - 1.0, 0.0        );
			let rb = ImVec2::new( i as f32 * note_size.x - 1.0, ctx.size.y );
			ctx.add_line( lt, rb, self.color_line, 1.0 );
		}
	}

	unsafe fn draw_notes( &self, ctx: &mut imutil::DrawContext, notes: &Vec<irgen::FlatNote>, note_size: ImVec2, color: u32 ) {
		use imgui::*;

		for note in notes.iter() {
			let nnum = match note.nnum {
				Some( v ) => v,
				None      => continue,
			};

			let x0 = ImVec2::new( note.bgn.to_float() as f32 * note_size.x,       (127 - nnum) as f32 * note_size.y );
			let x1 = ImVec2::new( note.end.to_float() as f32 * note_size.x - 1.0, (128 - nnum) as f32 * note_size.y );
			ctx.add_rect_filled( x0, x1, color, 0.0, !0 );

			let dur = note.end - note.bgn;
			SetCursorPos( &x0 );
			Dummy( &ImVec2::new( dur.to_float() as f32 * note_size.x - 1.0, note_size.y ) );
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
						misc::idiv( note.bgn.y, note.bgn.x ),
						misc::imod( note.bgn.y, note.bgn.x ),
						note.bgn.x,
					) );
					Text( c_str!( " duration = {}/{}", dur.y, dur.x ) );
				EndTooltip();
			}
		}
	}

	unsafe fn draw_time_bar( &mut self, ctx: &mut imutil::DrawContext, note_size: ImVec2, loc: f32 ) -> i32 {
		use imgui::*;
		let mut count = 0;

		for i in 0 .. self.loc_end.to_int() + 1 {
			SetCursorPos( &ImVec2::new( (i as f32 - 0.5) * note_size.x, 0.0 ) );
			if InvisibleButton( c_str!( "time_bar##{}", i ), &ImVec2::new( note_size.x, ctx.size.y ) ) {
				self.player.seek( ratio::Ratio::new( i * 2, 1 ) ).unwrap_or( () );
				count = cmp::max( count, JACK_FRAME_WAIT );
			}
		}

		let lt = ImVec2::new( loc * note_size.x - 1.0, 0.0                 );
		let rb = ImVec2::new( loc * note_size.x - 1.0, 128.0 * note_size.y );
		ctx.add_line( lt, rb, self.color_time_bar, 1.0 );

		count
	}
}

fn compile_task( file: &str, tx: window::MessageSender<UiMessage> ) -> Result<(), Box<error::Error>> {
	loop {
		let mut buf = String::new();
		fs::File::open( file )?.read_to_string( &mut buf )?;

		let compile = || -> Result<_, misc::Error> {
			let tree = parser::parse( &buf )?;
			let irgen = irgen::Generator::new( &tree );
			let mut migen = midi::Generator::new();
			let mut irs = Vec::new();
			for i in 0 .. 16 {
				let ir = irgen.generate( &format!( "out.{}", i ) )?.unwrap_or( Vec::new() );
				migen = migen.add_score( i, &ir );
				irs.push( ir );
			}
			let evs = migen.generate();
			Ok( (irs, evs) )
		};
		let msg = match compile() {
			Ok ( (irs, evs) ) => {
				UiMessage::Data( irs, evs )
			},
			Err( e ) => {
				let (row, col) = misc::text_row_col( &buf[0 .. e.loc] );
				UiMessage::Text( format!( "error at ({}, {}): {}", row, col, e.msg ) )
			},
		};
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
		imutil::set_scale( 1.5, 1.0, 13.0, font );

		let mut window = window::Window::new( Ui::new( "memol" )? );
		let tx = window.create_sender();
		thread::spawn( move || compile_task( &args.free[0], tx ).unwrap() );
		window.event_loop();

		Ok( () )
	};
	if let Err( e ) = f() {
		println!( "error: {}", e.description() );
	}
}
