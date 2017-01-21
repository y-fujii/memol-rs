// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
#![feature( untagged_unions )]
extern crate getopts;
extern crate gl;
extern crate glutin;
extern crate memol;
mod imgui;
mod imutil;
mod renderer;
use std::*;
use std::io::prelude::*;
use imgui::ImVec2;
use memol::*;


const JACK_FRAME_WAIT: i32 = 12;


struct Ui {
	irs: Vec<Vec<memol::irgen::FlatNote>>,
	player: Box<player::Player>,
	channel: i32,
	follow: bool,
	color_line: u32,
	color_time_bar: u32,
	color_chromatic: u32,
	color_note_top: u32,
	color_note_sub: u32,
}

impl imutil::Ui for Ui {
	fn draw( &mut self ) -> i32 {
		unsafe { self.draw_all() }
	}
}

impl Ui {
	fn new( player: Box<player::Player>, irs: Vec<Vec<memol::irgen::FlatNote>> ) -> Self {
		Ui {
			irs: irs,
			player: player,
			channel: 0,
			follow: false,
			color_line:      imutil::srgb_gamma( 0.5, 0.5, 0.5, 1.0 ),
			color_time_bar:  imutil::srgb_gamma( 0.0, 0.0, 0.0, 1.0 ),
			color_chromatic: imutil::srgb_gamma( 0.9, 0.9, 0.9, 1.0 ),
			color_note_top:  imutil::srgb_gamma( 0.1, 0.3, 0.4, 1.0 ),
			color_note_sub:  imutil::srgb_gamma( 0.7, 0.9, 1.0, 1.0 ),
		}
	}

	unsafe fn draw_all( &mut self ) -> i32 {
		use imgui::*;

		let mut count = 0;
		let mut ch_hovered = None;
		let loc = (self.player.location() / 2).to_float() as f32;
		let loc_end = self.irs.iter()
			.flat_map( |v| v.iter() )
			.map( |v| v.end )
			.max()
			.unwrap_or( ratio::Ratio::new( 0, 1 ) );

		SetNextWindowPos( &ImVec2::zero(), SetCond_Once );
		Begin( c_str!( "Transport" ), &mut true, WindowFlags_NoResize | WindowFlags_NoTitleBar );
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
				self.player.seek( loc_end * 2 ).unwrap_or( () );
				count = cmp::max( count, JACK_FRAME_WAIT );
			}

			SameLine( 0.0, -1.0 );
			for i in 0 .. self.irs.len() as i32 {
				RadioButton1( c_str!( "##{}", i ), &mut self.channel, i );
				if IsItemHovered() {
					ch_hovered = Some( i );
				}
				SameLine( 0.0, 0.0 );
			}
		End();

		let ch = ch_hovered.unwrap_or( self.channel );

		imutil::begin_root( WindowFlags_HorizontalScrollbar );
			let ctx = imutil::DrawContext::new();
			let note_size = ImVec2::new( (ctx.size.y / 8.0).ceil(), ctx.size.y / 128.0 );
			if self.follow {
				SetScrollX( loc * note_size.x - ctx.size.x / 2.0 );
			}
			else {
				count = cmp::max( count, self.drag_scroll() );
			}

			let mut ctx = imutil::DrawContext::new();
			let loc_end = loc_end.to_float() as f32;
			self.draw_background( &mut ctx, note_size, loc_end );
			for (i, ir) in self.irs.iter().enumerate() {
				if i != ch as usize {
					self.draw_notes( &mut ctx, ir, note_size, self.color_note_sub );
				}
			}
			self.draw_notes( &mut ctx, &self.irs[ch as usize], note_size, self.color_note_top );
			count = cmp::max( count, self.draw_time_bar( &mut ctx, note_size, loc, loc_end ) );
		imutil::end_root();

		cmp::max( count, if self.player.is_playing() { 1 } else { 0 } )
	}

	unsafe fn drag_scroll( &self ) -> i32 {
		use imgui::*;
		let delta = GetMouseDragDelta( 1, -1.0 );
		SetScrollX( GetScrollX() + 0.25 * delta.x );
		if delta.x != 0.0 { 1 } else { 0 }
	}

	unsafe fn draw_background( &self, ctx: &mut imutil::DrawContext, note_size: ImVec2, loc_end: f32 ) {
		use imgui::*;

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

		for i in 0 .. loc_end.floor() as i32 + 1 {
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
						 0 => "C",
						 1 => "C+",
						 2 => "D",
						 3 => "D+",
						 4 => "E",
						 5 => "F",
						 6 => "F+",
						 7 => "G",
						 8 => "G+",
						 9 => "A",
						10 => "A+",
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

	unsafe fn draw_time_bar( &mut self, ctx: &mut imutil::DrawContext, note_size: ImVec2, loc: f32, loc_end: f32 ) -> i32 {
		use imgui::*;
		let mut count = 0;

		for i in 0 .. loc_end.floor() as i64 + 1 {
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

fn main() {
	let f = || -> Result<(), Box<error::Error>> {
		let opts = getopts::Options::new();
		let args = opts.parse( env::args().skip( 1 ) )?;
		if args.free.len() != 1 {
			return Err( getopts::Fail::UnexpectedArgument( String::new() ).into() );
		}

		let mut buf = String::new();
		fs::File::open( &args.free[0] )?.read_to_string( &mut buf )?;
		let tree = parser::parse( &buf )?;
		let irgen = irgen::Generator::new( &tree );
		let mut migen = midi::Generator::new();
		let mut irs = Vec::new();
		for i in 0 .. 16 {
			let ir = irgen.generate( &format!( "out.{}", i ) )?.unwrap_or( Vec::new() );
			migen = migen.add_score( i, &ir );
			irs.push( ir );
		}

		let io = unsafe { &mut *imgui::GetIO() };
		io.IniFilename = ptr::null();
		let font = include_bytes!( "../imgui/extra_fonts/Cousine-Regular.ttf" );
		imutil::set_scale( 1.5, 1.5, 13.0, font );

		let mut player = player::Player::new( "memol" )?;
		player.set_data( migen.generate() );
		let mut window = imutil::Window::new( Ui::new( player, irs ) );
		window.event_loop();

		Ok( () )
	};
	if let Err( e ) = f() {
		println!( "error: {}", e.description() );
	}
}
