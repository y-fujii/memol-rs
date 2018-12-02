// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use clipboard;
use clipboard::ClipboardProvider;
use imgui::*;
use imutil;
use memol::misc;
use memol::midi;
use memol::generator;


pub enum Event {
	Seek( f32 ),
	NoteOn( i64 ),
	NoteClear,
}

pub struct PianoRoll {
	pub events: Vec<Event>,
	pub scroll_ratio: f32,
	pedal: bool,
	on_notes: [bool; 128],
	copying_notes: Vec<i64>,
	dragging: bool,
	time_scale: f32,
	line_width: f32,
	use_sharp: bool,
	clipboard: Option<clipboard::ClipboardContext>,
	color_line_0: u32,
	color_line_1: u32,
	color_note_0: u32,
	color_note_1: u32,
	color_hovered: u32,
}

impl PianoRoll {
	pub fn new() -> Self {
		Self {
			events: Vec::new(),
			scroll_ratio: 0.0,
			pedal: false,
			on_notes: [false; 128],
			copying_notes: Vec::new(),
			dragging: false,
			time_scale: 24.0,
			line_width: 0.25,
			use_sharp: false,
			clipboard: clipboard::ClipboardProvider::new().ok(),
			color_line_0:  imutil::pack_color( imutil::srgb_linear_to_gamma( ImVec4::new( 0.10, 0.10, 0.10, 0.50 ) ) ),
			color_line_1:  imutil::pack_color( imutil::srgb_linear_to_gamma( ImVec4::new( 0.10, 0.10, 0.10, 0.25 ) ) ),
			color_note_0:  imutil::pack_color( imutil::srgb_linear_to_gamma( ImVec4::new( 0.10, 0.10, 0.10, 1.00 ) ) ),
			color_note_1:  imutil::pack_color( imutil::srgb_linear_to_gamma( ImVec4::new( 0.80, 0.40, 0.10, 1.00 ) ) ),
			color_hovered: imutil::pack_color( imutil::srgb_linear_to_gamma( ImVec4::new( 1.00, 0.60, 0.30, 0.50 ) ) ),
		}
	}

	pub fn handle_midi_inputs<'a, T: Iterator<Item=&'a midi::Event>>( &mut self, events: T ) {
		for ev in events {
			match ev.msg[0] & 0xf0 {
				0x80 => {
					self.on_notes[ev.msg[1] as usize] = false;
				},
				0x90 => {
					self.on_notes[ev.msg[1] as usize] = true;
					if self.pedal {
						self.copying_notes.push( ev.msg[1] as i64 );
					}
				},
				0xb0 => {
					if ev.msg[1] == 64 {
						self.pedal = ev.msg[2] >= 64;
						if !self.pedal && !self.copying_notes.is_empty() {
							self.copy_notes_to_clipboard();
						}
						self.copying_notes.clear();
					}
				},
				_    => (),
			}
		}
	}

	pub unsafe fn draw( &mut self, ir: &generator::ScoreIr, time_len: f32, time_cur: f32, follow: bool, size: ImVec2 ) {
		let content_h = size.y - get_style().ScrollbarSize;
		let unit = content_h / 128.0;
		let content_w = unit * self.time_scale * (time_len + 1.0);
		let content_size = ImVec2::new( content_w, content_h );

		SetNextWindowContentSize( &content_size );
		BeginChild( c_str!( "piano_roll" ), &size, false, ImGuiWindowFlags_HorizontalScrollbar as i32 );
			if self.dragging | IsMouseDragging( 1, -1.0 ) {
				self.dragging = !IsMouseReleased( 1 );
				let a = 15.0 * get_io().DeltaTime;
				SetScrollX( GetScrollX() + a * GetMouseDragDelta( 1, -1.0 ).x );
			}
			else if self.copying_notes.is_empty() && IsMouseReleased( 1 ) {
				let x = (GetMousePos().x - GetWindowContentRegionMin().x) / (unit * self.time_scale) - 0.5;
				self.events.push( Event::Seek( x ) );
			}
			else if follow {
				let next = (time_cur + 0.5) * self.time_scale * unit - (1.0 / 6.0) * size.x;
				let a = f32::exp( -2.0 * get_io().DeltaTime );
				SetScrollX( a * GetScrollX() + (1.0 - a) * next );
			}

			let mut ctx = imutil::DrawContext::new( unit, ImVec2::new( unit * self.time_scale * 0.5, 0.0 ) );
			self.draw_indicator( &mut ctx, time_len );
			self.draw_background( &mut ctx, time_len );
			self.draw_notes( &mut ctx, &ir, time_cur, self.color_note_0, self.color_note_1 );
			self.draw_time_bar( &mut ctx, time_cur );

			self.scroll_ratio = if GetScrollMaxX() > 0.0 { GetScrollX() / GetScrollMaxX() } else { 0.5 };
		EndChild();
	}

	unsafe fn draw_indicator( &mut self, ctx: &mut imutil::DrawContext, time_len: f32 ) {
		for y in 0 .. 128 {
			let v0 = ImVec2::new( self.time_scale * 0.0     , y as f32 );
			let v1 = ImVec2::new( self.time_scale * time_len, y as f32 + 1.0 );
			ctx.add_dummy( v0, v1 );
			if IsItemHovered( 0 ) {
				if IsMouseClicked( 0, false ) {
					self.copying_notes.push( y );
					self.events.push( Event::NoteOn( y ) );
				}
				if !self.copying_notes.is_empty() && IsMouseReleased( 1 ) {
					self.copy_notes_to_clipboard();
					self.copying_notes.clear();
					self.events.push( Event::NoteClear );
				}
			}
			if IsItemHovered( 0 ) || self.on_notes[y as usize] {
				ctx.add_rect_filled( v0, v1, self.color_hovered, 0.0, !0 );
			}
		}
		if self.copying_notes.len() > 0 {
			BeginTooltip();
				imutil::show_text( &Self::note_symbols( &self.copying_notes, self.use_sharp ) );
			EndTooltip();
		}
	}

	unsafe fn draw_background( &mut self, ctx: &mut imutil::DrawContext, time_len: f32 ) {
		// vertical lines.
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

		// bold horizontal lines.
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

		// thin horizontal lines.
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

	unsafe fn draw_notes( &self, ctx: &mut imutil::DrawContext, ir: &generator::ScoreIr, time_cur: f32, color_0: u32, color_1: u32 ) {
		for note in ir.iter() {
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

			ctx.add_dummy( x0, x1 );
			if IsItemHovered( 0 ) {
				BeginTooltip();
					imutil::show_text( &format!( "     note = {}", Self::note_symbol( nnum, self.use_sharp ) ) );
					imutil::show_text( &format!( "gate time = {} + {}/{}",
						misc::idiv( note.t0.y, note.t0.x ),
						misc::imod( note.t0.y, note.t0.x ),
						note.t0.x,
					) );
					let dt = note.t1 - note.t0;
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

	fn note_symbols( notes: &[i64], use_sharp: bool ) -> String {
		let mut buf = String::new();
		let mut n0 = notes[0];
		for &n1 in notes.iter() {
			let sym = if n1 <= n0 { ">" } else { "<" };
			for _ in 0 .. (n1 - n0).abs() / 12 {
				buf.push_str( sym );
			}
			let sym = Self::note_symbol( n1, use_sharp );
			let sym = if n1 <= n0 { sym.to_lowercase() } else { sym.to_uppercase() };
			buf.push_str( &sym );
			n0 = n1;
		}
		buf
	}

	fn note_symbol( n: i64, use_sharp: bool ) -> &'static str {
		let syms = if use_sharp {
			[ "c", "c+", "d", "d+", "e", "f", "f+", "g", "g+", "a", "a+", "b" ]
		}
		else {
			[ "c", "d-", "d", "e-", "e", "f", "g-", "g", "a-", "a", "b-", "b" ]
		};
		syms[misc::imod( n, 12 ) as usize]
	}

	fn copy_notes_to_clipboard( &mut self ) {
		if let Some( ref mut clipboard ) = self.clipboard {
			clipboard.set_contents( Self::note_symbols( &self.copying_notes, self.use_sharp ) ).ok();
		}
	}
}
