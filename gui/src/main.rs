// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
#![feature( untagged_unions )]
extern crate getopts;
extern crate gl;
extern crate glutin;
extern crate memol;
mod imgui;
mod renderer;
use std::*;
use std::io::prelude::*;
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

		unsafe {
			Self::begin_root( WindowFlags_HorizontalScrollbar );
				let size = GetWindowSize();
				let note_size = ImVec2::new( (size.y / 8.0).ceil(), size.y / 128.0 );
				redraw |= self.drag_scroll();
				self.draw_background( note_size, time_max.to_float() as f32 );
				self.draw_notes( note_size );
			Self::end_root();

			SetNextWindowPos( &ImVec2::zero(), SetCond_Once );
			Begin( c_str!( "Transport" ), &mut true, WindowFlags_NoResize | WindowFlags_NoTitleBar );
				Button( c_str!( "<<" ), &ImVec2::zero() );
				SameLine( 0.0, -1.0 );
				Button( c_str!( "Play" ), &ImVec2::zero() );
				SameLine( 0.0, -1.0 );
				Button( c_str!( "Stop" ), &ImVec2::zero() );
				SameLine( 0.0, -1.0 );
				Button( c_str!( ">>" ), &ImVec2::zero() );
				SameLine( 0.0, -1.0 );
				Checkbox( c_str!( "Follow" ), &mut self.follow );
				for i in 0 .. self.show.len() {
					SameLine( 0.0, -1.0 );
					Checkbox( c_str!( "{}", i ), &mut self.show[i] );
				}
			End();
		}
		redraw
	}

	unsafe fn drag_scroll( &self ) -> bool {
		use imgui::*;
		let delta = GetMouseDragDelta( 0, 1.0 );
		SetScrollX( GetScrollX() + 0.25 * delta.x );
		SetScrollY( GetScrollY() + 0.25 * delta.y );
		delta.x != 0.0 || delta.y != 0.0
	}

	unsafe fn draw_background( &self, note_size: imgui::ImVec2, t1: f32 ) {
		use imgui::*;
		let dl = &mut *GetWindowDrawList();
		let orig = Self::get_origin();

		for i in 0i32 .. (128 + 11) / 12 {
			dl.AddLine(
				&(orig + ImVec2::new( 0.0,              (i * 12) as f32 * note_size.y )),
				&(orig + ImVec2::new( t1 * note_size.x, (i * 12) as f32 * note_size.y )),
				self.color_line, 1.0,
			);
			for j in [ 1, 3, 6, 8, 10 ].iter() {
				dl.AddRectFilled(
					&(orig + ImVec2::new( 0.0,              (i * 12 + j + 0) as f32 * note_size.y )),
					&(orig + ImVec2::new( t1 * note_size.x, (i * 12 + j + 1) as f32 * note_size.y )),
					self.color_chromatic, 0.0, !0,
				);
			}
		}

		for i in 0 .. t1.floor() as i32 + 1 {
			dl.AddLine(
				&(orig + ImVec2::new( i as f32 * note_size.x - 1.0, 0.0                 )),
				&(orig + ImVec2::new( i as f32 * note_size.x - 1.0, 128.0 * note_size.y )),
				self.color_line, 1.0,
			);
		}
	}

	unsafe fn draw_notes( &self, note_size: imgui::ImVec2 ) {
		use imgui::*;
		let dl = &mut *GetWindowDrawList();
		let orig = Self::get_origin();

		let mut i = 0;
		for note in self.irs.iter().flat_map( |v| v.iter() ).flat_map( |v| v.iter() ) {
			let nnum = match note.nnum {
				Some( v ) => v,
				None      => continue,
			};

			let x0 = ImVec2::new( note.bgn.to_float() as f32 * note_size.x,       (nnum + 0) as f32 * note_size.y );
			let x1 = ImVec2::new( note.end.to_float() as f32 * note_size.x - 1.0, (nnum + 1) as f32 * note_size.y );
			dl.AddRectFilled( &(orig + x0), &(orig + x1), self.color_note, 0.0, !0 );

			let dur = note.end - note.bgn;
			SetCursorPos( &x0 );
			InvisibleButton( c_str!( "note##{}", i ), &ImVec2::new( dur.to_float() as f32 * note_size.x - 1.0, note_size.y ) );
			if IsItemHovered() {
				BeginTooltip();
				Text( c_str!( "note number = {}", nnum ) );
				Text( c_str!( "  gate time = {}/{}", note.bgn.y, note.bgn.x ) );
				Text( c_str!( "   duration = {}/{}", dur.y, dur.x ) );
				EndTooltip();
			}

			i += 1;
		}
	}

	unsafe fn begin_root( flags: imgui::ImGuiWindowFlags ) {
		use imgui::*;
		let size = (*GetIO()).DisplaySize;
		let rounding = (*GetStyle()).WindowRounding;
		let padding  = (*GetStyle()).WindowPadding;
		PushStyleVar( StyleVar::WindowRounding as i32, 0.0 );
		PushStyleVar1( StyleVar::WindowPadding as i32, &ImVec2::zero() );
		SetNextWindowPos( &ImVec2::zero(), SetCond_Always );
		SetNextWindowSize( &size, SetCond_Always );
		Begin1(
			c_str!( "root" ), &mut true, &size, 0.0,
			WindowFlags_NoMove | WindowFlags_NoResize | WindowFlags_NoBringToFrontOnFocus |
			WindowFlags_NoTitleBar | flags
		);
		PushStyleVar( StyleVar::WindowRounding as i32, rounding );
		PushStyleVar1( StyleVar::WindowPadding as i32, &padding );
	}

	unsafe fn end_root() {
		use imgui::*;
		PopStyleVar( 2 );
		End();
		PopStyleVar( 2 );
	}

	unsafe fn get_origin() -> imgui::ImVec2 {
		use imgui::*;
		GetWindowPos() - ImVec2::new( GetScrollX(), GetScrollY() )
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

		unsafe { &mut *imgui::GetIO() }.IniFilename = ptr::null();
		let mut window = Window::new( Ui::new( irs ) );
		window.event_loop();

		Ok( () )
	}().unwrap();
}
