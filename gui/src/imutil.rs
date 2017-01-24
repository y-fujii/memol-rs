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

pub struct MessageSender<T> {
	tx: sync::mpsc::Sender<T>,
	proxy: glutin::WindowProxy,
}

impl<T> MessageSender<T> {
	pub fn send( &self, msg: T ) -> Result<(), sync::mpsc::SendError<T>> {
		self.tx.send( msg )?;
		self.proxy.wakeup_event_loop();
		Ok( () )
	}
}

pub trait Ui<T> {
	fn on_draw( &mut self ) -> i32;
	fn on_message( &mut self, T );
}

pub struct Window<T, U: Ui<T>> {
	window: glutin::Window,
	renderer: renderer::Renderer,
	ui: U,
	tx: sync::mpsc::Sender<T>,
	rx: sync::mpsc::Receiver<T>,
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

impl<T, U: Ui<T>> Window<T, U> {
	pub fn new( ui: U ) -> Self {
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

		let (tx, rx) = sync::mpsc::channel();
		Window {
			window: window,
			renderer: renderer::Renderer::new(),
			ui: ui,
			tx: tx,
			rx: rx,
		}
	}

	pub fn create_sender( &self ) -> MessageSender<T> {
		MessageSender{
			tx: self.tx.clone(),
			proxy: self.window.create_window_proxy(),
		}
	}

	pub fn event_loop( &mut self ) {
		loop {
			let mut n = 2;
			while n > 0 {
				//for ev in self.window.poll_events() {
				while let Some( ev ) = self.window.poll_events().next() {
					if self.handle_event( &ev ) {
						return;
					}
					n = cmp::max( n, 2 );
				}

				unsafe { imgui::NewFrame() };
				n = cmp::max( n - 1, self.ui.on_draw() );
				unsafe { gl::Clear( gl::COLOR_BUFFER_BIT ); }
				self.renderer.render();
				self.window.swap_buffers().unwrap();
			}

			let ev = self.window.wait_events().next().unwrap();
			if self.handle_event( &ev ) {
				return;
			}
		}
	}

	fn handle_event( &mut self, ev: &glutin::Event ) -> bool {
		match *ev {
			glutin::Event::Closed => true,
			glutin::Event::Awakened => {
				match self.rx.try_recv() {
					Err( _ ) => (),
					Ok ( v ) => self.ui.on_message( v ),
				};
				false
			},
			_ => {
				self.renderer.handle_event( ev );
				false
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
		let io = imgui::get_io();
		let mut cfg = imgui::ImFontConfig::new();
		cfg.FontDataOwnedByAtlas = false;
		(*io.Fonts).AddFontFromMemoryTTF(
			font.as_ptr() as *mut os::raw::c_void,
			font.len() as i32, font_size * s, &cfg, ptr::null(),
		);

		let style = imgui::get_style();
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
		let size = get_io().DisplaySize;
		let rounding = get_style().WindowRounding;
		let padding  = get_style().WindowPadding;
		PushStyleVar( StyleVar::WindowRounding as i32, 0.0 );
		PushStyleVar1( StyleVar::WindowPadding as i32, &ImVec2::zero() );
		SetNextWindowPos( &ImVec2::zero(), SetCond_Always );
		SetNextWindowSize( &size, SetCond_Always );
		Begin1(
			c_str!( "root" ), &mut true, &size, 0.0,
			WindowFlags_NoMove | WindowFlags_NoResize |
			WindowFlags_NoBringToFrontOnFocus | WindowFlags_NoTitleBar | flags
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
