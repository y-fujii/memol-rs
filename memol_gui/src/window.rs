// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::imgui;
use crate::renderer;
use glutin::event;
use glutin::event_loop;
use glutin::window;
use std::*;

pub struct Window<T: 'static + fmt::Debug> {
    context: *mut imgui::ImGuiContext,
    looper: Option<event_loop::EventLoop<T>>,
    window: glutin::WindowedContext<glutin::PossiblyCurrent>,
    renderer: renderer::Renderer,
    timer: time::SystemTime,
    background: Option<imgui::ImVec4>,
    on_message: Box<dyn 'static + FnMut(T) -> i32>,
    on_draw: Box<dyn 'static + FnMut() -> i32>,
    on_file_dropped: Box<dyn 'static + FnMut(&path::PathBuf) -> i32>,
}

impl<T: fmt::Debug> Drop for Window<T> {
    fn drop(&mut self) {
        let io = imgui::get_io();
        renderer::destroy_font_texture(unsafe { &mut *io.Fonts });
        unsafe { imgui::DestroyContext(self.context) };
    }
}

impl<T: fmt::Debug> Window<T> {
    pub fn new() -> Result<Self, Box<dyn error::Error>> {
        let context = unsafe { imgui::CreateContext(ptr::null_mut()) };

        let io = imgui::get_io();
        io.IniFilename = ptr::null();
        use event::VirtualKeyCode;
        io.KeyMap[imgui::ImGuiKey_Tab as usize] = VirtualKeyCode::Tab as i32;
        io.KeyMap[imgui::ImGuiKey_LeftArrow as usize] = VirtualKeyCode::Left as i32;
        io.KeyMap[imgui::ImGuiKey_RightArrow as usize] = VirtualKeyCode::Right as i32;
        io.KeyMap[imgui::ImGuiKey_UpArrow as usize] = VirtualKeyCode::Up as i32;
        io.KeyMap[imgui::ImGuiKey_DownArrow as usize] = VirtualKeyCode::Down as i32;
        io.KeyMap[imgui::ImGuiKey_PageUp as usize] = VirtualKeyCode::PageUp as i32;
        io.KeyMap[imgui::ImGuiKey_PageDown as usize] = VirtualKeyCode::PageDown as i32;
        io.KeyMap[imgui::ImGuiKey_Home as usize] = VirtualKeyCode::Home as i32;
        io.KeyMap[imgui::ImGuiKey_End as usize] = VirtualKeyCode::End as i32;
        io.KeyMap[imgui::ImGuiKey_Delete as usize] = VirtualKeyCode::Delete as i32;
        io.KeyMap[imgui::ImGuiKey_Backspace as usize] = VirtualKeyCode::Back as i32;
        io.KeyMap[imgui::ImGuiKey_Enter as usize] = VirtualKeyCode::Return as i32;
        io.KeyMap[imgui::ImGuiKey_Escape as usize] = VirtualKeyCode::Escape as i32;
        io.KeyMap[imgui::ImGuiKey_Space as usize] = VirtualKeyCode::Space as i32;
        io.KeyMap[imgui::ImGuiKey_A as usize] = VirtualKeyCode::A as i32;
        io.KeyMap[imgui::ImGuiKey_C as usize] = VirtualKeyCode::C as i32;
        io.KeyMap[imgui::ImGuiKey_V as usize] = VirtualKeyCode::V as i32;
        io.KeyMap[imgui::ImGuiKey_X as usize] = VirtualKeyCode::X as i32;
        io.KeyMap[imgui::ImGuiKey_Y as usize] = VirtualKeyCode::Y as i32;
        io.KeyMap[imgui::ImGuiKey_Z as usize] = VirtualKeyCode::Z as i32;

        let looper = event_loop::EventLoop::new_user_event();
        let window = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::GlThenGles {
                opengl_version: (3, 3),
                opengles_version: (3, 0),
            })
            .with_gl_profile(glutin::GlProfile::Core)
            .with_vsync(true)
            .build_windowed(window::WindowBuilder::new(), &looper)?;
        let window = unsafe { window.make_current() }.map_err(|(_, e)| e)?;
        gl::load_with(|s| window.get_proc_address(s) as *const _);
        let renderer = renderer::Renderer::new(window.get_api() != glutin::Api::OpenGl);

        Ok(Window {
            context: context,
            looper: Some(looper),
            window: window,
            renderer: renderer,
            timer: time::SystemTime::now(),
            background: None,
            on_message: Box::new(|_| 0),
            on_draw: Box::new(|| 0),
            on_file_dropped: Box::new(|_| 0),
        })
    }

    pub fn set_background(&mut self, col: imgui::ImVec4) {
        self.background = Some(col);
    }

    pub fn on_message<U: 'static + FnMut(T) -> i32>(&mut self, f: U) {
        self.on_message = Box::new(f);
    }

    pub fn on_draw<U: 'static + FnMut() -> i32>(&mut self, f: U) {
        self.on_draw = Box::new(f);
    }

    pub fn on_file_dropped<U: 'static + FnMut(&path::PathBuf) -> i32>(&mut self, f: U) {
        self.on_file_dropped = Box::new(f);
    }

    pub fn hidpi_factor(&self) -> f64 {
        self.window.window().hidpi_factor()
    }

    pub fn update_font(&mut self) {
        let io = imgui::get_io();
        renderer::update_font_texture(unsafe { &mut *io.Fonts })
    }

    pub fn create_proxy(&self) -> event_loop::EventLoopProxy<T> {
        self.looper.as_ref().unwrap().create_proxy()
    }

    pub fn run(mut self) {
        self.handle_window_event(event::WindowEvent::Resized(self.window.window().inner_size()));

        let mut n = 1;
        let looper = self.looper.take().unwrap();
        looper.run(move |ev, _, flow| {
            match ev {
                event::Event::EventsCleared => {
                    if n > 0 {
                        if let Some(col) = self.background {
                            self.renderer.clear(col);
                        }

                        let timer = mem::replace(&mut self.timer, time::SystemTime::now());
                        let delta = self.timer.duration_since(timer).unwrap();
                        let delta = delta.as_secs() as f32 + delta.subsec_nanos() as f32 * 1e-9;
                        // DeltaTime == 0.0 cause repeating clicks.
                        imgui::get_io().DeltaTime = f32::max(delta, f32::EPSILON);
                        unsafe { imgui::NewFrame() };
                        n = cmp::max(n - 1, (self.on_draw)());
                        unsafe { imgui::EndFrame() };

                        unsafe { imgui::Render() };
                        self.renderer
                            .render(unsafe { &*imgui::GetDrawData() }, imgui::get_io().DisplaySize);
                        if let Err(_) = self.window.swap_buffers() {
                            *flow = event_loop::ControlFlow::Exit;
                            return;
                        }

                        if (0..3).any(|i| imgui::get_io().MouseDown[i]) {
                            n = cmp::max(n, 1);
                        }
                        *flow = if n > 0 {
                            event_loop::ControlFlow::Poll
                        } else {
                            event_loop::ControlFlow::Wait
                        }
                    }
                }
                event::Event::WindowEvent {
                    window_id: id,
                    event: ev,
                    ..
                } => {
                    if id == self.window.window().id() {
                        match ev {
                            event::WindowEvent::CloseRequested => {
                                *flow = event_loop::ControlFlow::Exit;
                                return;
                            }
                            event::WindowEvent::DroppedFile(ref path) => {
                                n = cmp::max(n, (self.on_file_dropped)(path));
                                n = cmp::max(n, 1);
                            }
                            ev => {
                                if self.handle_window_event(ev) {
                                    imgui::get_io().DeltaTime = f32::EPSILON;
                                    unsafe { imgui::NewFrame() };
                                    n = cmp::max(n, (self.on_draw)());
                                    n = cmp::max(n, 1);
                                    unsafe { imgui::EndFrame() };
                                }
                            }
                        }
                    }
                }
                event::Event::UserEvent(ev) => {
                    n = cmp::max(n, (self.on_message)(ev));
                    n = cmp::max(n, 1);
                }
                _ => (),
            }
            *flow = event_loop::ControlFlow::Wait;
        });
    }

    fn handle_window_event(&mut self, ev: event::WindowEvent) -> bool {
        use event::*;

        let io = imgui::get_io();
        match ev {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(code),
                        ..
                    },
                ..
            } => {
                let pressed = state == ElementState::Pressed;
                match code {
                    VirtualKeyCode::LControl | VirtualKeyCode::RControl => io.KeyCtrl = pressed,
                    VirtualKeyCode::LShift | VirtualKeyCode::RShift => io.KeyShift = pressed,
                    VirtualKeyCode::LAlt | VirtualKeyCode::RAlt => io.KeyAlt = pressed,
                    VirtualKeyCode::LWin | VirtualKeyCode::RWin => io.KeySuper = pressed,
                    c => io.KeysDown[c as usize] = pressed,
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;
                match button {
                    MouseButton::Left => io.MouseDown[0] = pressed,
                    MouseButton::Right => io.MouseDown[1] = pressed,
                    MouseButton::Middle => io.MouseDown[2] = pressed,
                    _ => return false,
                }
            }
            WindowEvent::ReceivedCharacter(c) => {
                unsafe { io.AddInputCharacter(c as u32) };
            }
            WindowEvent::MouseWheel {
                delta: event::MouseScrollDelta::LineDelta(x, y),
                phase: event::TouchPhase::Moved,
                ..
            } => {
                let scale = 1.0 / 5.0;
                io.MouseWheelH = scale * x;
                io.MouseWheel = scale * y;
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::PixelDelta(delta),
                phase: TouchPhase::Moved,
                ..
            } => {
                // XXX
                let delta = delta.to_physical(self.window.window().hidpi_factor());
                let scale = 1.0 / (5.0 * unsafe { imgui::GetFontSize() });
                io.MouseWheelH = scale * delta.x as f32;
                io.MouseWheel = scale * delta.y as f32;
            }
            WindowEvent::CursorMoved { position: pos, .. } => {
                // XXX
                let pos = pos.to_physical(self.window.window().hidpi_factor());
                io.MousePos.x = pos.x as f32;
                io.MousePos.y = pos.y as f32;
            }
            WindowEvent::Resized(logical) => {
                let physical = logical.to_physical(self.window.window().hidpi_factor());
                io.DisplaySize.x = physical.width as f32;
                io.DisplaySize.y = physical.height as f32;
                // Wayland needs to resize context manually.
                self.window.resize(physical);
            }
            WindowEvent::HiDpiFactorChanged(factor) => {
                let logical = self.window.window().inner_size();
                let physical = logical.to_physical(factor);
                io.DisplaySize.x = physical.width as f32;
                io.DisplaySize.y = physical.height as f32;
                // Wayland needs to resize context manually.
                self.window.resize(physical);
            }
            _ => {
                return false;
            }
        }
        true
    }
}
