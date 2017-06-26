// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
extern crate gl;
extern crate glutin;
use imgui;
use renderer;
use std::*;


pub trait Ui<T> {
	fn on_draw( &mut self ) -> i32 { 0 }
	fn on_file_dropped( &mut self, &path::PathBuf ) -> i32 { 0 }
	fn on_message( &mut self, T ) -> i32 { 0 }
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

pub struct Window<T, U: Ui<T>> {
	window: glutin::Window,
	renderer: renderer::Renderer,
	timer: time::SystemTime,
	ui: U,
	tx: sync::mpsc::Sender<T>,
	rx: sync::mpsc::Receiver<T>,
}

impl<T, U: Ui<T>> Window<T, U> {
	pub fn new( ui: U ) -> Self {
		let io = imgui::get_io();
		io.KeyMap[imgui::ImGuiKey_Tab        as usize] = glutin::VirtualKeyCode::Tab as i32;
		io.KeyMap[imgui::ImGuiKey_LeftArrow  as usize] = glutin::VirtualKeyCode::Left as i32;
		io.KeyMap[imgui::ImGuiKey_RightArrow as usize] = glutin::VirtualKeyCode::Right as i32;
		io.KeyMap[imgui::ImGuiKey_UpArrow    as usize] = glutin::VirtualKeyCode::Up as i32;
		io.KeyMap[imgui::ImGuiKey_DownArrow  as usize] = glutin::VirtualKeyCode::Down as i32;
		io.KeyMap[imgui::ImGuiKey_PageUp     as usize] = glutin::VirtualKeyCode::PageUp as i32;
		io.KeyMap[imgui::ImGuiKey_PageDown   as usize] = glutin::VirtualKeyCode::PageDown as i32;
		io.KeyMap[imgui::ImGuiKey_Home       as usize] = glutin::VirtualKeyCode::Home as i32;
		io.KeyMap[imgui::ImGuiKey_End        as usize] = glutin::VirtualKeyCode::End as i32;
		io.KeyMap[imgui::ImGuiKey_Delete     as usize] = glutin::VirtualKeyCode::Delete as i32;
		io.KeyMap[imgui::ImGuiKey_Backspace  as usize] = glutin::VirtualKeyCode::Back as i32;
		io.KeyMap[imgui::ImGuiKey_Enter      as usize] = glutin::VirtualKeyCode::Return as i32;
		io.KeyMap[imgui::ImGuiKey_Escape     as usize] = glutin::VirtualKeyCode::Escape as i32;
		io.KeyMap[imgui::ImGuiKey_A          as usize] = glutin::VirtualKeyCode::A as i32;
		io.KeyMap[imgui::ImGuiKey_C          as usize] = glutin::VirtualKeyCode::C as i32;
		io.KeyMap[imgui::ImGuiKey_V          as usize] = glutin::VirtualKeyCode::V as i32;
		io.KeyMap[imgui::ImGuiKey_X          as usize] = glutin::VirtualKeyCode::X as i32;
		io.KeyMap[imgui::ImGuiKey_Y          as usize] = glutin::VirtualKeyCode::Y as i32;
		io.KeyMap[imgui::ImGuiKey_Z          as usize] = glutin::VirtualKeyCode::Z as i32;

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
			timer: time::UNIX_EPOCH,
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

	pub fn event_loop( &mut self ) -> result::Result<(), Box<error::Error>> {
		let (x, y) = self.window.get_inner_size().unwrap_or( (640, 480) );
		let mut n = 1 + self.handle_event( &glutin::Event::Resized( x, y ) );
		loop {
			while n > 0 {
				//for ev in self.window.poll_events() {
				while let Some( ev ) = self.window.poll_events().next() {
					if let glutin::Event::Closed = ev {
						return Ok( () );
					}
					n = cmp::max( n, 1 + self.handle_event( &ev ) );
				}

				let timer = time::SystemTime::now();
				let delta = timer.duration_since( self.timer )?;
				self.timer = timer;
				imgui::get_io().DeltaTime = delta.as_secs() as f32 * 1e3 + delta.subsec_nanos() as f32 / 1e6;
				unsafe { imgui::NewFrame() };
				n = cmp::max( n, 1 + self.ui.on_draw() );
				unsafe { imgui::Render() };

				unsafe { gl::Clear( gl::COLOR_BUFFER_BIT ); }
				self.renderer.render();
				self.window.swap_buffers()?;
				n -= 1;
			}

			let ev = self.window.wait_events().next().unwrap();
			if let glutin::Event::Closed = ev {
				return Ok( () );
			}
			n = cmp::max( n, 1 + self.handle_event( &ev ) );
		}
	}

	fn handle_event( &mut self, ev: &glutin::Event ) -> i32 {
		use glutin::*;
		let io = imgui::get_io();
		match *ev {
			Event::KeyboardInput( s, _, Some( code ) ) => {
				let pressed = s == ElementState::Pressed;
				match code {
					VirtualKeyCode::LControl | VirtualKeyCode::RControl => io.KeyCtrl  = pressed,
					VirtualKeyCode::LShift   | VirtualKeyCode::RShift   => io.KeyShift = pressed,
					VirtualKeyCode::LAlt     | VirtualKeyCode::RAlt     => io.KeyAlt   = pressed,
					c => io.KeysDown[c as usize] = pressed,
				}
			},
			Event::MouseInput( s, k ) => {
				let pressed = s == ElementState::Pressed;
				match k {
					MouseButton::Left   => io.MouseDown[0] = pressed,
					MouseButton::Right  => io.MouseDown[1] = pressed,
					MouseButton::Middle => io.MouseDown[2] = pressed,
					_ => (),
				}
			}
			Event::ReceivedCharacter( c ) => {
				unsafe { io.AddInputCharacter( c as u16 ) };
			},
			Event::MouseWheel( MouseScrollDelta::LineDelta ( _, y ), TouchPhase::Moved ) |
			Event::MouseWheel( MouseScrollDelta::PixelDelta( _, y ), TouchPhase::Moved ) => {
				io.MouseWheel = y;
			},
			Event::MouseMoved( x, y ) => {
				io.MousePos = imgui::ImVec2::new(
					x as f32 / io.DisplayFramebufferScale.x,
					y as f32 / io.DisplayFramebufferScale.y
				);
			},
			Event::Resized( x, y ) => {
				io.DisplaySize.x = x as f32 / io.DisplayFramebufferScale.x;
				io.DisplaySize.y = y as f32 / io.DisplayFramebufferScale.y;
			},
			Event::DroppedFile( ref path ) => {
				self.ui.on_file_dropped( path );
			},
			_ => (),
		}

		let mut n = 1;
		while let Ok( v ) = self.rx.try_recv() {
			n = cmp::max( n, self.ui.on_message( v ) );
		}
		n
	}
}
