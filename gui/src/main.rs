extern crate getopts;
extern crate gl;
extern crate glutin;
#[macro_use]
extern crate rust_imgui;
extern crate memol;
mod renderer;
use std::*;
use std::io::prelude::*;
use rust_imgui as imgui;
use memol::*;


struct Ui {
	irs: Vec<Option<Vec<memol::irgen::FlatNote>>>,
	follow: bool,
	show: Vec<bool>,
	color_line: u32,
	color_chromatic: u32,
	color_note: u32,
}

struct Window {
	window: glutin::Window,
	renderer: renderer::Renderer,
	ui: Ui,
}

impl Ui {
	fn new( irs: Vec<Option<Vec<memol::irgen::FlatNote>>> ) -> Self {
		Ui {
			show: vec![true; irs.len()],
			irs: irs,
			follow: true,
			color_line: 0xffe0e0e0,
			color_chromatic: 0xfff0f0f0,
			color_note: 0xff90a080,
		}
	}

	fn draw( &mut self ) -> bool {
		use imgui::*;
		let mut redraw = false;

		let time_max = self.irs.iter()
			.flat_map( |v| v.iter() )
			.flat_map( |v| v.iter() )
			.map( |v| v.end )
			.max()
			.unwrap_or( ratio::Ratio::new( 0, 1 ) );

		Self::begin_root( ImGuiWindowFlags_HorizontalScrollbar );
			let size = Self::get_window_size();
			let note_size = vec2( (size.y / 8.0).ceil(), size.y / 128.0 );
			redraw |= self.drag_scroll();
			self.draw_background( note_size, time_max.to_float() as f32 );
			self.draw_notes( note_size );
		Self::end_root();

		set_next_window_pos( ImVec2::zero(), ImGuiSetCond::Once );
		begin( imstr!( "Transport" ), &mut true, ImGuiWindowFlags_NoResize | ImGuiWindowFlags_NoTitleBar );
			button( imstr!( "<<" ), ImVec2::zero() );
			same_line();
			button( imstr!( "Play" ), ImVec2::zero() );
			same_line();
			button( imstr!( "Stop" ), ImVec2::zero() );
			same_line();
			button( imstr!( ">>" ), ImVec2::zero() );
			same_line();
			checkbox( imstr!( "Follow" ), &mut self.follow );
			for i in 0 .. self.show.len() {
				same_line();
				checkbox( imstr!( "{}", i ), &mut self.show[i] );
			}
		end();
		redraw
	}

	fn drag_scroll( &self ) -> bool {
		use imgui::*;

		let mut delta = unsafe { mem::uninitialized() };
		get_mouse_drag_delta( &mut delta, 0, 1.0 );
		set_scroll_x( get_scroll_x() + 0.25 * delta.x );
		set_scroll_y( get_scroll_y() + 0.25 * delta.y );
		delta.x != 0.0 || delta.y != 0.0
	}

	fn draw_background( &self, note_size: imgui::ImVec2, t1: f32 ) {
		use imgui::*;
		let dl = get_window_draw_list().unwrap();
		let orig = Self::get_origin();

		for i in 0 .. (128 + 11) / 12 {
			dl.add_line(
				vec2( orig.x,                    orig.y + (i * 12) as f32 * note_size.y ),
				vec2( orig.x + t1 * note_size.x, orig.y + (i * 12) as f32 * note_size.y ),
				self.color_line, 1.0,
			);
			for j in [ 1, 3, 6, 8, 10 ].iter() {
				dl.add_rect_filled_simple(
					vec2( orig.x,                    orig.y + (i * 12 + j + 0) as f32 * note_size.y ),
					vec2( orig.x + t1 * note_size.x, orig.y + (i * 12 + j + 1) as f32 * note_size.y ),
					self.color_chromatic,
				);
			}
		}

		for i in 0 .. t1.floor() as i32 + 1 {
			dl.add_line(
				vec2( orig.x + i as f32 * note_size.x - 1.0, orig.y                       ),
				vec2( orig.x + i as f32 * note_size.x - 1.0, orig.y + 128.0 * note_size.y ),
				self.color_line, 1.0,
			);
		}
	}

	fn draw_notes( &self, note_size: imgui::ImVec2 ) {
		use imgui::*;
		let dl = get_window_draw_list().unwrap();
		let orig = Self::get_origin();

		let mut i = 0;
		for note in self.irs.iter().flat_map( |v| v.iter() ).flat_map( |v| v.iter() ) {
			let nnum = match note.nnum {
				Some( v ) => v,
				None      => continue,
			};

			let x0 = vec2( note.bgn.to_float() as f32 * note_size.x,       (nnum + 0) as f32 * note_size.y );
			let x1 = vec2( note.end.to_float() as f32 * note_size.x - 1.0, (nnum + 1) as f32 * note_size.y );
			dl.add_rect_filled_simple(
				vec2( orig.x + x0.x, orig.y + x0.y ),
				vec2( orig.x + x1.x, orig.y + x1.y ),
				self.color_note,
			);

			let dur = note.end - note.bgn;
			set_cursor_pos( x0 );
			invisible_button( imstr!( "note##{}", i ), vec2( dur.to_float() as f32 * note_size.x - 1.0, note_size.y ) );
			if is_item_hovered() {
				begin_tooltip();
				text( imstr!( "note number = {}", nnum ) );
				text( imstr!( "  gate time = {}/{}", note.bgn.y, note.bgn.x ) );
				text( imstr!( "   duration = {}/{}", dur.y, dur.x ) );
				end_tooltip();
			}

			i += 1;
		}
	}

	fn begin_root( flags: imgui::ImGuiWindowFlags ) {
		use imgui::*;

		let size = get_io().display_size;
		let rounding = get_style().unwrap().window_rounding;
		let padding  = get_style().unwrap().window_padding;
		push_style_var( ImGuiStyleVar::WindowRounding, 0.0 );
		push_style_var_vec( ImGuiStyleVar::WindowPadding, ImVec2::zero() );
		set_next_window_pos( ImVec2::zero(), ImGuiSetCond::Always );
		set_next_window_size( size, ImGuiSetCond::Always );
		begin2(
			imstr!( "root" ), &mut true, size, 0.0,
			ImGuiWindowFlags_NoMove | ImGuiWindowFlags_NoResize |
			ImGuiWindowFlags_NoBringToFrontOnFocus |
			ImGuiWindowFlags_NoTitleBar | flags
		);
		push_style_var( ImGuiStyleVar::WindowRounding, rounding );
		push_style_var_vec( ImGuiStyleVar::WindowPadding, padding );
	}

	fn end_root() {
		imgui::pop_style_var( 2 );
		imgui::end();
		imgui::pop_style_var( 2 );
	}

	fn get_window_pos() -> imgui::ImVec2 {
		let mut pos = unsafe { mem::uninitialized() };
		imgui::get_window_pos( &mut pos );
		pos
	}

	fn get_window_size() -> imgui::ImVec2 {
		let mut size = unsafe { mem::uninitialized() };
		imgui::get_window_size( &mut size );
		size
	}

	fn get_origin() -> imgui::ImVec2 {
		imgui::vec2(
			Self::get_window_pos().x - imgui::get_scroll_x(),
			Self::get_window_pos().y - imgui::get_scroll_y(),
		)
	}
}

impl Window {
	fn new( ui: Ui ) -> Self {
		let window = glutin::WindowBuilder::new()
			.with_gl_profile( glutin::GlProfile::Core )
			.with_vsync()
			.build()
			.unwrap();

		unsafe {
			window.make_current().unwrap();
			gl::load_with( |s| window.get_proc_address( s ) as *const os::raw::c_void );
			gl::ClearColor( 1.0, 1.0, 1.0, 1.0 );
		}

		Window {
			window: window,
			renderer: renderer::Renderer::new(),
			ui: ui,
		}
	}

	// XXX
	fn event_loop( &mut self ) {
		for _ in 0 .. 2 {
			self.renderer.new_frame( self.window.get_inner_size().unwrap(), self.window.hidpi_factor() );
			self.ui.draw();
		}
		unsafe { gl::Clear( gl::COLOR_BUFFER_BIT ); }
		self.renderer.render();
		self.window.swap_buffers().unwrap();

		for ev in self.window.wait_events() {
			self.renderer.handle_event( &ev );
			if let glutin::Event::Closed = ev {
				return;
			}

			loop {
				for ev in self.window.poll_events() {
					self.renderer.handle_event( &ev );
					if let glutin::Event::Closed = ev {
						return;
					}
				}

				self.renderer.new_frame( self.window.get_inner_size().unwrap(), self.window.hidpi_factor() );
				let redraw = self.ui.draw();
				unsafe { gl::Clear( gl::COLOR_BUFFER_BIT ); }
				self.renderer.render();
				self.window.swap_buffers().unwrap();
				if !redraw {
					break;
				}
			}
		}
	}
}

fn main() {
	|| -> Result<(), Box<error::Error>> {
		let opts = getopts::Options::new();
		let args = opts.parse( env::args().skip( 1 ) )?;
		if args.free.len() != 1 {
			return misc::error( "" );
		}

		let mut buf = String::new();
		fs::File::open( &args.free[0] )?.read_to_string( &mut buf )?;
		let tree = parser::parse( &buf )?;
		let irgen = irgen::Generator::new( &tree );
		let mut irs = Vec::new();
		for ch in 0 .. 16 {
			irs.push( irgen.generate( &format!( "out.{}", ch ) )? );
		}

		imgui::get_io().ini_filename = ptr::null();
		let mut window = Window::new( Ui::new( irs ) );
		window.event_loop();

		Ok( () )
	}().unwrap();
}
