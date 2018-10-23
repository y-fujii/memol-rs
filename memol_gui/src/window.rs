// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
extern crate gl;
extern crate glutin;
use std::*;
use glutin::GlContext;
use imgui;
use renderer;


pub trait Ui<T> {
	fn on_draw( &mut self ) -> i32 { 0 }
	fn on_file_dropped( &mut self, &path::PathBuf ) -> i32 { 0 }
	fn on_message( &mut self, T ) -> i32 { 0 }
}

pub struct MessageSender<T> {
	tx: sync::mpsc::Sender<T>,
	proxy: glutin::EventsLoopProxy,
}

impl<T> MessageSender<T> {
	pub fn send( &self, msg: T ) {
		self.tx.send( msg ).unwrap();
		self.proxy.wakeup().unwrap();
	}
}

pub struct Window<T, U: Ui<T>> {
	context: *mut imgui::ImGuiContext,
	looper: glutin::EventsLoop,
	window: glutin::GlWindow,
	renderer: renderer::Renderer,
	timer: time::SystemTime,
	ui: U,
	tx: sync::mpsc::Sender<T>,
	rx: sync::mpsc::Receiver<T>,
}

impl<T, U: Ui<T>> Drop for Window<T, U> {
	fn drop( &mut self ) {
		unsafe { imgui::DestroyContext( self.context ) };
	}
}

impl<T, U: Ui<T>> Window<T, U> {
	pub fn new( ui: U ) -> Result<Self, Box<error::Error>> {
		let context = unsafe { imgui::CreateContext( ptr::null_mut() ) };

		let io = imgui::get_io();
		io.IniFilename = ptr::null();
		io.KeyMap[imgui::ImGuiKey_Tab        as usize] = glutin::VirtualKeyCode::Tab      as i32;
		io.KeyMap[imgui::ImGuiKey_LeftArrow  as usize] = glutin::VirtualKeyCode::Left     as i32;
		io.KeyMap[imgui::ImGuiKey_RightArrow as usize] = glutin::VirtualKeyCode::Right    as i32;
		io.KeyMap[imgui::ImGuiKey_UpArrow    as usize] = glutin::VirtualKeyCode::Up       as i32;
		io.KeyMap[imgui::ImGuiKey_DownArrow  as usize] = glutin::VirtualKeyCode::Down     as i32;
		io.KeyMap[imgui::ImGuiKey_PageUp     as usize] = glutin::VirtualKeyCode::PageUp   as i32;
		io.KeyMap[imgui::ImGuiKey_PageDown   as usize] = glutin::VirtualKeyCode::PageDown as i32;
		io.KeyMap[imgui::ImGuiKey_Home       as usize] = glutin::VirtualKeyCode::Home     as i32;
		io.KeyMap[imgui::ImGuiKey_End        as usize] = glutin::VirtualKeyCode::End      as i32;
		io.KeyMap[imgui::ImGuiKey_Delete     as usize] = glutin::VirtualKeyCode::Delete   as i32;
		io.KeyMap[imgui::ImGuiKey_Backspace  as usize] = glutin::VirtualKeyCode::Back     as i32;
		io.KeyMap[imgui::ImGuiKey_Enter      as usize] = glutin::VirtualKeyCode::Return   as i32;
		io.KeyMap[imgui::ImGuiKey_Escape     as usize] = glutin::VirtualKeyCode::Escape   as i32;
		io.KeyMap[imgui::ImGuiKey_Space      as usize] = glutin::VirtualKeyCode::Space    as i32;
		io.KeyMap[imgui::ImGuiKey_A          as usize] = glutin::VirtualKeyCode::A        as i32;
		io.KeyMap[imgui::ImGuiKey_C          as usize] = glutin::VirtualKeyCode::C        as i32;
		io.KeyMap[imgui::ImGuiKey_V          as usize] = glutin::VirtualKeyCode::V        as i32;
		io.KeyMap[imgui::ImGuiKey_X          as usize] = glutin::VirtualKeyCode::X        as i32;
		io.KeyMap[imgui::ImGuiKey_Y          as usize] = glutin::VirtualKeyCode::Y        as i32;
		io.KeyMap[imgui::ImGuiKey_Z          as usize] = glutin::VirtualKeyCode::Z        as i32;

		let looper = glutin::EventsLoop::new();
		let window = {
			let builder = glutin::WindowBuilder::new();
			let context = glutin::ContextBuilder::new()
				.with_gl( glutin::GlRequest::GlThenGles{
					opengl_version:   (3, 3),
					opengles_version: (3, 0),
				} )
				.with_gl_profile( glutin::GlProfile::Core )
				.with_vsync( true );
			glutin::GlWindow::new( builder, context, &looper )?
		};
		unsafe {
			window.make_current()?;
			gl::load_with( |s| window.get_proc_address( s ) as *const os::raw::c_void );
		}
		let renderer = renderer::Renderer::new( window.get_api() != glutin::Api::OpenGl );

		let (tx, rx) = sync::mpsc::channel();
		Ok( Window {
			context: context,
			looper: looper,
			window: window,
			renderer: renderer,
			timer: time::SystemTime::now(),
			ui: ui,
			tx: tx,
			rx: rx,
		} )
	}

	pub fn ui_mut( &mut self ) -> &mut U {
		&mut self.ui
	}

	pub fn hidpi_factor( &self ) -> f64 {
		self.window.get_hidpi_factor()
	}

	pub fn update_font( &mut self ) {
		self.renderer.update_font()
	}

	pub fn create_sender( &self ) -> MessageSender<T> {
		MessageSender{
			tx: self.tx.clone(),
			proxy: self.looper.create_proxy(),
		}
	}

	pub fn event_loop( &mut self ) -> result::Result<(), Box<error::Error>> {
		let io = imgui::get_io();

		let mut n: i32 = 1;
		let mut events = Vec::new();
		events.push( glutin::Event::WindowEvent{
			window_id: self.window.id(),
			event: glutin::WindowEvent::Resized( glutin::dpi::LogicalSize::new( f64::NAN, f64::NAN ) ),
		} );
		loop {
			if n > 0 {
				self.looper.poll_events( |ev|
					if let glutin::Event::DeviceEvent{ .. } = ev {} else {
						events.push( ev );
					}
				);
			}
			else {
				self.looper.run_forever( |ev| {
					if let glutin::Event::DeviceEvent{ .. } = ev {
						glutin::ControlFlow::Continue
					}
					else {
						events.push( ev );
						glutin::ControlFlow::Break
					}
				} );
			}

			if events.is_empty() {
				n = cmp::max( n - 1, self.handle_ui()? );
			}
			for ev in events.drain( .. ) {
				let k = match self.handle_event( &ev ) {
					Some( k ) => k,
					None      => return Ok( () ),
				};
				n = cmp::max( n - 1, k );
				n = cmp::max( n, self.handle_ui()? );
			}

			if (0 .. 3).any( |i| io.MouseDown[i] ) {
				n = cmp::max( n, 1 );
			}

			self.renderer.render();
			self.window.swap_buffers()?;
		}
	}

	fn handle_ui( &mut self ) -> result::Result<i32, Box<error::Error>> {
		let timer = mem::replace( &mut self.timer, time::SystemTime::now() );
		let delta = self.timer.duration_since( timer )?;
		let delta = delta.as_secs() as f32 + delta.subsec_nanos() as f32 * 1e-9;
		// DeltaTime == 0.0 cause repeating clicks.
		imgui::get_io().DeltaTime = f32::max( delta, f32::EPSILON );

		unsafe { imgui::NewFrame() };
		let n = self.ui.on_draw();
		unsafe { imgui::Render() };
		Ok( n )
	}

	fn handle_event( &mut self, ev: &glutin::Event ) -> Option<i32> {
		use glutin::*;

		let mut n = 1;

		let io = imgui::get_io();
		if let Event::WindowEvent{ event: ref ev, .. } = *ev {
			match ev {
				WindowEvent::KeyboardInput{ input: KeyboardInput{ state, virtual_keycode: Some( code ), .. }, .. } => {
					let pressed = *state == ElementState::Pressed;
					match code {
						VirtualKeyCode::LControl | VirtualKeyCode::RControl => io.KeyCtrl  = pressed,
						VirtualKeyCode::LShift   | VirtualKeyCode::RShift   => io.KeyShift = pressed,
						VirtualKeyCode::LAlt     | VirtualKeyCode::RAlt     => io.KeyAlt   = pressed,
						VirtualKeyCode::LWin     | VirtualKeyCode::RWin     => io.KeySuper = pressed,
						c => io.KeysDown[*c as usize] = pressed,
					}
				},
				WindowEvent::MouseInput{ state, button, .. } => {
					let pressed = *state == ElementState::Pressed;
					match button {
						MouseButton::Left   => io.MouseDown[0] = pressed,
						MouseButton::Right  => io.MouseDown[1] = pressed,
						MouseButton::Middle => io.MouseDown[2] = pressed,
						_ => (),
					}
				},
				WindowEvent::ReceivedCharacter( c ) => {
					unsafe { io.AddInputCharacter( *c as u16 ) };
				},
				WindowEvent::MouseWheel{ delta: MouseScrollDelta::LineDelta( x, y ), phase: TouchPhase::Moved, .. } => {
					let scale = 1.0 / 5.0;
					io.MouseWheelH = scale * x;
					io.MouseWheel  = scale * y;
				},
				WindowEvent::MouseWheel{ delta: MouseScrollDelta::PixelDelta( delta ), phase: TouchPhase::Moved, .. } => {
					// XXX
					let delta = delta.to_physical( self.window.get_hidpi_factor() );
					let scale = 1.0 / (5.0 * unsafe { imgui::GetFontSize() });
					io.MouseWheelH = scale * delta.x as f32;
					io.MouseWheel  = scale * delta.y as f32;
				},
				WindowEvent::CursorMoved{ position: pos, .. } => {
					// XXX
					let pos = pos.to_physical( self.window.get_hidpi_factor() );
					io.MousePos.x = pos.x as f32;
					io.MousePos.y = pos.y as f32;
				},
				WindowEvent::Resized( _ ) |
				WindowEvent::HiDpiFactorChanged( _ ) => {
					// XXX
					let size = self.window
						.get_inner_size()
						.unwrap()
						.to_physical( self.window.get_hidpi_factor() );
					io.DisplaySize.x = size.width  as f32;
					io.DisplaySize.y = size.height as f32;
					// Wayland needs to resize context manually.
					self.window.context().resize( size );
				},
				WindowEvent::DroppedFile( ref path ) => {
					n = cmp::max( n, self.ui.on_file_dropped( path ) );
				},
				WindowEvent::CloseRequested => {
					return None;
				},
				_ => (),
			}
		}

		while let Ok( v ) = self.rx.try_recv() {
			n = cmp::max( n, self.ui.on_message( v ) );
		}
		Some( n )
	}
}
