// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
extern crate gl;
extern crate glutin;
use imgui;
use renderer;
use std::*;
#[allow( unused_imports )]
use memol::*; // c_str!()


pub struct DrawContext<'a> {
	pub draw_list: &'a mut imgui::ImDrawList,
	pub origin: imgui::ImVec2,
	pub size: imgui::ImVec2,
}

pub trait Ui {
	fn draw( &mut self ) -> bool;
}

pub struct Window<T: Ui> {
	window: glutin::Window,
	renderer: renderer::Renderer,
	ui: T,
}

impl<'a> DrawContext<'a> {
	pub fn new() -> DrawContext<'static> {
		use imgui::*;

		unsafe {
			let size = GetWindowSize();
			DrawContext{
				draw_list: &mut *GetWindowDrawList(),
				origin: GetWindowPos() - ImVec2::new( GetScrollX(), GetScrollY() ),
				size: size,
			}
		}
	}

	pub fn add_line( &mut self, a: imgui::ImVec2, b: imgui::ImVec2, col: u32, thickness: f32 ) {
		unsafe {
			self.draw_list.AddLine( &(self.origin + a), &(self.origin + b), col, thickness );
		}
	}

	pub fn add_rect_filled( &mut self, a: imgui::ImVec2, b: imgui::ImVec2, col: u32, rounding: f32, flags: i32 ) {
		unsafe {
			self.draw_list.AddRectFilled( &(self.origin + a), &(self.origin + b), col, rounding, flags );
		}
	}
}

impl<T: Ui> Window<T> {
	pub fn new( ui: T ) -> Self {
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
	pub fn event_loop( &mut self ) {
		for _ in 0 .. 2 {
			self.renderer.new_frame( self.window.get_inner_size().unwrap() );
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

				self.renderer.new_frame( self.window.get_inner_size().unwrap() );
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

// XXX
pub fn srgb_gamma( r: f32, g: f32, b: f32, a: f32 ) -> u32 {
	(((r.powf( 1.0 / 2.2 ) * 255.0) as u32) <<  0) |
	(((g.powf( 1.0 / 2.2 ) * 255.0) as u32) <<  8) |
	(((b.powf( 1.0 / 2.2 ) * 255.0) as u32) << 16) |
	(((a                   * 255.0) as u32) << 24)
}

pub fn set_scale( s: f32, r: f32, font_size: f32, font: &[u8] ) {
    unsafe {
        let io = &mut *imgui::GetIO();
        let mut cfg = imgui::ImFontConfig::new();
        cfg.FontDataOwnedByAtlas = false;
        (*io.Fonts).AddFontFromMemoryTTF(
            font.as_ptr() as *mut os::raw::c_void,
            font.len() as i32, font_size * s, &cfg, ptr::null()
        );

        let style = &mut *imgui::GetStyle();
        style.WindowPadding *= s;
        style.WindowMinSize *= s;
        style.WindowRounding *= r;
        style.WindowTitleAlign *= s;
        style.ChildWindowRounding *= r;
        style.FramePadding *= s;
        style.FrameRounding *= r;
        style.ItemSpacing *= s;
        style.ItemInnerSpacing *= s;
        style.TouchExtraPadding *= s;
        style.IndentSpacing *= s;
        style.ColumnsMinSpacing *= s;
        style.ScrollbarSize *= s;
        style.ScrollbarRounding *= r;
        style.GrabMinSize *= s;
        style.GrabRounding *= r;
        style.ButtonTextAlign *= s;
        style.DisplayWindowPadding *= s;
        style.DisplaySafeAreaPadding *= s;
        style.CurveTessellationTol *= s;
    }
}

pub fn begin_root( flags: imgui::ImGuiWindowFlags ) {
    use imgui::*;
    unsafe {
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
}

pub fn end_root() {
    use imgui::*;
    unsafe {
        PopStyleVar( 2 );
        End();
        PopStyleVar( 2 );
    }
}
