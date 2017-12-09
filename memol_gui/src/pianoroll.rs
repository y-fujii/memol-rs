// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::*;
use imgui::*;
use imutil;
use memol::*;


pub struct PianoRoll {
	pub scroll: f32,
	dragging: bool,
	time_scale: f32,
	line_width: f32,
	color_line_0: u32,
	color_line_1: u32,
	color_note_0: u32,
	color_note_1: u32,
}

impl PianoRoll {
	pub fn new() -> Self {
		Self {
			scroll: 0.0,
			dragging: false,
			time_scale: 24.0,
			line_width: 0.25,
			color_line_0: imutil::pack_color( imutil::srgb_linear_to_gamma( ImVec4::new( 0.10, 0.10, 0.10, 0.50 ) ) ),
			color_line_1: imutil::pack_color( imutil::srgb_linear_to_gamma( ImVec4::new( 0.10, 0.10, 0.10, 0.25 ) ) ),
			color_note_0: imutil::pack_color( imutil::srgb_linear_to_gamma( ImVec4::new( 0.10, 0.10, 0.10, 1.00 ) ) ),
			color_note_1: imutil::pack_color( imutil::srgb_linear_to_gamma( ImVec4::new( 0.80, 0.40, 0.10, 1.00 ) ) ),
		}
	}

	pub unsafe fn draw( &mut self, ir: &scoregen::Ir, time_len: f32, time_cur: f32, follow: bool, size: ImVec2 ) -> Option<f32> {
		let content_h = size.y - get_style().ScrollbarSize;
		let unit = content_h / 128.0;
		let content_w = unit * self.time_scale * (time_len + 1.0);
		let content_size = ImVec2::new( content_w, content_h );

		SetNextWindowContentSize( &content_size );
		BeginChild( c_str!( "piano_roll" ), &size, false, ImGuiWindowFlags_HorizontalScrollbar as i32 );
			let clicked = InvisibleButton( c_str!( "background" ), &content_size );

			self.dragging |= IsItemActive() && IsMouseDragging( 0, -1.0 );

			let mut seek = None;
			if self.dragging {
				SetScrollX( GetScrollX() + 0.25 * GetMouseDragDelta( 0, -1.0 ).x );
			}
			else if clicked {
				let x = (GetMousePos().x - GetWindowContentRegionMin().x) / (unit * self.time_scale) - 0.5;
				seek = Some( x );
			}
			else if follow {
				let next = (time_cur + 0.5) * self.time_scale * unit - (1.0 / 6.0) * size.x;
				SetScrollX( (31.0 / 32.0) * GetScrollX() + (1.0 / 32.0) * next );
			}

			self.dragging &= !IsMouseReleased( 0 );

			let mut ctx = imutil::DrawContext::new( unit, ImVec2::new( unit * self.time_scale * 0.5, 0.0 ) );
			self.draw_background( &mut ctx, time_len );
			self.draw_notes( &mut ctx, &ir, time_cur, self.color_note_0, self.color_note_1 );
			self.draw_time_bar( &mut ctx, time_cur );

			self.scroll = if GetScrollMaxX() > 0.0 { GetScrollX() / GetScrollMaxX() } else { 0.5 };
		EndChild();

		seek
	}

	unsafe fn draw_background( &self, ctx: &mut imutil::DrawContext, time_len: f32 ) {
		for i in 0 .. time_len.floor() as i32 + 1 {
			let ys = [
				(43 - 24, 57 - 24),
				(43     , 57     ),
				(64     , 77     ),
				(64 + 24, 77 + 24),
			];
			for &(y0, y1) in ys.iter() {
				let v0 = ImVec2::new( self.time_scale * i as f32, y0 as f32 + 0.5 );
				let v1 = ImVec2::new( self.time_scale * i as f32, y1 as f32 + 0.5 );
				ctx.add_line( v0, v1, self.color_line_0, self.line_width );
			}
		}

		let ys = [
			43 - 24, 47 - 24, 50 - 24, 53 - 24, 57 - 24,
			43,      47,      50,      53,      57,
			64,      67,      71,      74,      77,
			64 + 24, 67 + 24, 71 + 24, 74 + 24, 77 + 24,
		];
		for &y in ys.iter() {
			let v0 = ImVec2::new( self.time_scale * 0.0      - 0.5 * self.line_width, y as f32 + 0.5 );
			let v1 = ImVec2::new( self.time_scale * time_len + 0.5 * self.line_width, y as f32 + 0.5 );
			ctx.add_line( v0, v1, self.color_line_0, self.line_width );
		}

		let ys = [
			36,      40,
			60,
			81,      84,
			81 + 24, 84 + 24,
		];
		for &y in ys.iter() {
			let v0 = ImVec2::new( self.time_scale * 0.0      - 0.5 * self.line_width, y as f32 + 0.5 );
			let v1 = ImVec2::new( self.time_scale * time_len + 0.5 * self.line_width, y as f32 + 0.5 );
			ctx.add_line( v0, v1, self.color_line_1, self.line_width );
		}
	}

	unsafe fn draw_notes( &self, ctx: &mut imutil::DrawContext, ir: &scoregen::Ir, time_cur: f32, color_0: u32, color_1: u32 ) {
		for note in ir.notes.iter() {
			let nnum = match note.nnum {
				Some( v ) => v,
				None      => continue,
			};

			let t0 = note.t0.to_float() as f32;
			let t1 = note.t1.to_float() as f32;
			let x0 = ImVec2::new( self.time_scale * t0, nnum as f32 + 0.0 );
			let x1 = ImVec2::new( self.time_scale * t1, nnum as f32 + 1.0 );
			let color = if t0 <= time_cur && time_cur <= t1 { color_1 } else { color_0 };
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

	unsafe fn draw_time_bar( &self, ctx: &mut imutil::DrawContext, time_cur: f32 ) {
		let v0 = ImVec2::new( self.time_scale * time_cur,   0.0 );
		let v1 = ImVec2::new( self.time_scale * time_cur, 128.0 );
		ctx.add_line( v0, v1, self.color_note_1, self.line_width );
	}
}
