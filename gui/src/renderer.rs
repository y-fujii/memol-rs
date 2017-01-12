use std::*;
use std::os::raw::{ c_void, c_char };
use gl;
use glutin;
use rust_imgui as imgui;


pub struct Renderer {
	pub font_texture: u32,
	pub program: u32,
	pub vao: u32,
	pub vbo: u32,
	pub ebo: u32,
}

impl Drop for Renderer {
	fn drop( &mut self ) {
		unsafe {
			gl::DeleteVertexArrays( 1, &self.vao );
			gl::DeleteBuffers( 1, &self.vbo );
			gl::DeleteBuffers( 1, &self.ebo );

			let mut shaders: [u32; 2] = mem::uninitialized();
			gl::GetAttachedShaders( self.program, shaders.len() as i32, ptr::null_mut(), shaders.as_mut_ptr() );
			gl::DeleteProgram( self.program );
			for s in shaders.iter() {
				gl::DeleteShader( *s );
			}

			gl::DeleteTextures( 1, &self.font_texture );
		}
		imgui::get_io().fonts.tex_id = ptr::null_mut();
		imgui::shutdown();
	}
}

const VERT_SHADER_CODE: &'static str = r#"
	#version 330
	uniform vec2 Scale;
	layout( location = 0 ) in vec2 pos;
	layout( location = 1 ) in vec2 uv;
	layout( location = 2 ) in vec4 color;
	out vec2 frag_uv;
	out vec4 frag_color;

	void main() {
		frag_uv = uv;
		frag_color = color;
		gl_Position = vec4( Scale * pos + vec2( -1.0, +1.0 ), 0.0, 1.0 );
	}
"#;

const FRAG_SHADER_CODE: &'static str = r#"
	#version 330
	uniform sampler2D Texture;
	in vec2 frag_uv;
	in vec4 frag_color;
	out vec4 out_color;

	void main() {
		out_color = frag_color * texture( Texture, frag_uv );
	}
"#;

unsafe fn compile_shader( ty: u32, code: &str ) -> u32 {
	let shader = gl::CreateShader( ty );
	let ptr = code.as_ptr() as *const i8;
	let len = code.len() as i32;
	gl::ShaderSource( shader, 1, &ptr, &len );
	gl::CompileShader( shader );
	let mut is_compiled = 0;
	gl::GetShaderiv( shader, gl::COMPILE_STATUS, &mut is_compiled );
	assert!( is_compiled != 0 );
	shader
}

impl Renderer {
	pub fn new() -> Self {
		let io = imgui::get_io();

		// key mapping.
		io.key_map[imgui::ImGuiKey::Tab        as usize] = glutin::VirtualKeyCode::Tab as i32;
		io.key_map[imgui::ImGuiKey::LeftArrow  as usize] = glutin::VirtualKeyCode::Left as i32;
		io.key_map[imgui::ImGuiKey::RightArrow as usize] = glutin::VirtualKeyCode::Right as i32;
		io.key_map[imgui::ImGuiKey::UpArrow    as usize] = glutin::VirtualKeyCode::Up as i32;
		io.key_map[imgui::ImGuiKey::DownArrow  as usize] = glutin::VirtualKeyCode::Down as i32;
		io.key_map[imgui::ImGuiKey::PageUp     as usize] = glutin::VirtualKeyCode::PageUp as i32;
		io.key_map[imgui::ImGuiKey::PageDown   as usize] = glutin::VirtualKeyCode::PageDown as i32;
		io.key_map[imgui::ImGuiKey::Home       as usize] = glutin::VirtualKeyCode::Home as i32;
		io.key_map[imgui::ImGuiKey::End        as usize] = glutin::VirtualKeyCode::End as i32;
		io.key_map[imgui::ImGuiKey::Delete     as usize] = glutin::VirtualKeyCode::Delete as i32;
		io.key_map[imgui::ImGuiKey::Backspace  as usize] = glutin::VirtualKeyCode::Back as i32;
		io.key_map[imgui::ImGuiKey::Enter      as usize] = glutin::VirtualKeyCode::Return as i32;
		io.key_map[imgui::ImGuiKey::Escape     as usize] = glutin::VirtualKeyCode::Escape as i32;
		io.key_map[imgui::ImGuiKey::A          as usize] = glutin::VirtualKeyCode::A as i32;
		io.key_map[imgui::ImGuiKey::C          as usize] = glutin::VirtualKeyCode::C as i32;
		io.key_map[imgui::ImGuiKey::V          as usize] = glutin::VirtualKeyCode::V as i32;
		io.key_map[imgui::ImGuiKey::X          as usize] = glutin::VirtualKeyCode::X as i32;
		io.key_map[imgui::ImGuiKey::Y          as usize] = glutin::VirtualKeyCode::Y as i32;
		io.key_map[imgui::ImGuiKey::Z          as usize] = glutin::VirtualKeyCode::Z as i32;

		let mut data = ptr::null_mut();
		let mut w = 0;
		let mut h = 0;
		let mut bpp = 0;
		io.fonts.get_tex_data_as_rgba32( &mut data, &mut w, &mut h, &mut bpp );

		let prog;
		let mut tex = 0;
		let mut vao = 0;
		let mut vbo = 0;
		let mut ebo = 0;
		unsafe {
			// font texture.
			gl::GenTextures( 1, &mut tex );
			gl::BindTexture( gl::TEXTURE_2D, tex );
			gl::TexParameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32 );
			gl::TexParameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );
			gl::TexImage2D( gl::TEXTURE_2D, 0, gl::RGBA as i32, w, h, 0, gl::RGBA, gl::UNSIGNED_BYTE, data as *const c_void );
			io.fonts.tex_id = mem::transmute( tex as usize );

			// shader program.
			let vert = compile_shader( gl::VERTEX_SHADER,   VERT_SHADER_CODE );
			let frag = compile_shader( gl::FRAGMENT_SHADER, FRAG_SHADER_CODE );
			prog = gl::CreateProgram();
			gl::AttachShader( prog, vert );
			gl::AttachShader( prog, frag );
			gl::LinkProgram( prog );

			// vertex objects.
			gl::GenVertexArrays( 1, &mut vao );
			gl::BindVertexArray( vao );

			gl::GenBuffers( 1, &mut vbo );
			gl::BindBuffer( gl::ARRAY_BUFFER, vbo );

			gl::GenBuffers( 1, &mut ebo );
			gl::BindBuffer( gl::ELEMENT_ARRAY_BUFFER, ebo );

			gl::EnableVertexAttribArray( 0 );
			gl::EnableVertexAttribArray( 1 );
			gl::EnableVertexAttribArray( 2 );

			let size = mem::size_of::<imgui::ImDrawVert>() as i32;
			gl::VertexAttribPointer( 0, 2, gl::FLOAT,         gl::FALSE, size,  0 as *const c_void );
			gl::VertexAttribPointer( 1, 2, gl::FLOAT,         gl::FALSE, size,  8 as *const c_void );
			gl::VertexAttribPointer( 2, 4, gl::UNSIGNED_BYTE, gl::TRUE,  size, 16 as *const c_void );

			gl::BindTexture( gl::TEXTURE_2D, 0 );
			gl::BindVertexArray( 0 );
			gl::BindBuffer( gl::ELEMENT_ARRAY_BUFFER, 0 );
			gl::BindBuffer( gl::ARRAY_BUFFER, 0 );
		}

		Renderer {
			font_texture: tex,
			program: prog,
			vao: vao,
			vbo: vbo,
			ebo: ebo,
		}
	}

	pub fn new_frame( &mut self, display_size: (u32, u32), scale: f32 ) {
		let io = imgui::get_io();
		io.display_size.x = display_size.0 as f32;
		io.display_size.y = display_size.1 as f32;
		io.display_framebuffer_scale.x = scale;
		io.display_framebuffer_scale.y = scale;
		imgui::new_frame();
	}

	pub fn handle_event( &mut self, ev: &glutin::Event ) {
		use glutin::*;

		let io = imgui::get_io();
		match *ev {
			Event::KeyboardInput( s, _, Some( code ) ) => {
				let pressed = imgui::cbool( s == ElementState::Pressed );
				match code {
					VirtualKeyCode::LControl | VirtualKeyCode::RControl => io.key_ctrl  = pressed,
					VirtualKeyCode::LShift   | VirtualKeyCode::RShift   => io.key_shift = pressed,
					VirtualKeyCode::LAlt     | VirtualKeyCode::RAlt     => io.key_alt   = pressed,
					c => io.keys_down[c as usize] = pressed,
				}
			},
			Event::MouseInput( s, k ) => {
				let pressed = imgui::cbool( s == ElementState::Pressed );
				match k {
					MouseButton::Left   => io.mouse_down[0] = pressed,
					MouseButton::Right  => io.mouse_down[1] = pressed,
					MouseButton::Middle => io.mouse_down[2] = pressed,
					_ => (),
				}
			}
			Event::ReceivedCharacter( c ) => {
				io.add_input_character( c as u16 );
			},
			Event::MouseWheel( MouseScrollDelta::LineDelta ( _, y ), TouchPhase::Moved ) |
			Event::MouseWheel( MouseScrollDelta::PixelDelta( _, y ), TouchPhase::Moved ) => {
				io.mouse_wheel = y;
			},
			Event::MouseMoved( x, y ) => {
				io.mouse_pos = imgui::vec2(
					x as f32 / io.display_framebuffer_scale.x,
					y as f32 / io.display_framebuffer_scale.y
				);
			},
			_ => (),
		}
	}

	pub fn render( &self ) {
		imgui::render();
		let io = imgui::get_io();
		let draw_data = imgui::get_draw_data().unwrap();

		unsafe {
			gl::Enable( gl::BLEND );
			gl::BlendEquation( gl::FUNC_ADD );
			gl::BlendFunc( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA );
			gl::Disable( gl::CULL_FACE );
			gl::Disable( gl::DEPTH_TEST );
			gl::Enable( gl::SCISSOR_TEST );
			gl::ActiveTexture( gl::TEXTURE0 );

			gl::Viewport( 0, 0,
				(io.display_size.x * io.display_framebuffer_scale.x) as i32,
				(io.display_size.y * io.display_framebuffer_scale.y) as i32,
			);
			gl::UseProgram( self.program );
			gl::Uniform1i( gl::GetUniformLocation( self.program, "Texture\0".as_ptr() as *const c_char ), 0 );
			gl::Uniform2f( gl::GetUniformLocation( self.program,   "Scale\0".as_ptr() as *const c_char ),
				2.0 / io.display_size.x, -2.0 / io.display_size.y );
			gl::BindVertexArray( self.vao );

			for i in 0 .. draw_data.cmd_lists_count {
				let cmd_list = &**draw_data.cmd_lists.offset( i as isize );

				gl::BindBuffer( gl::ARRAY_BUFFER, self.vbo );
				gl::BufferData(
					gl::ARRAY_BUFFER,
					cmd_list.vtx_buffer.size as isize * mem::size_of::<imgui::ImDrawVert>() as isize,
					cmd_list.vtx_buffer.data as *const c_void,
					gl::STREAM_DRAW
				);

				gl::BindBuffer( gl::ELEMENT_ARRAY_BUFFER, self.ebo );
				gl::BufferData(
					gl::ELEMENT_ARRAY_BUFFER,
					cmd_list.idx_buffer.size as isize * mem::size_of::<imgui::ImDrawIdx>() as isize,
					cmd_list.idx_buffer.data as *const c_void,
					gl::STREAM_DRAW
				);

				let mut offset = 0;
				for i in 0 .. cmd_list.cmd_buffer.size {
					let cmd = &cmd_list.cmd_buffer[i as usize];
					if let Some( cb ) = cmd.user_callback {
						cb( cmd_list, cmd );
					}
					else {
						gl::BindTexture( gl::TEXTURE_2D, cmd.texture_id as u32 );
						gl::Scissor(
							(io.display_framebuffer_scale.x * cmd.clip_rect.x) as i32,
							(io.display_framebuffer_scale.y * (io.display_size.y - cmd.clip_rect.w)) as i32,
							(io.display_framebuffer_scale.x * (cmd.clip_rect.z   - cmd.clip_rect.x)) as i32,
							(io.display_framebuffer_scale.y * (cmd.clip_rect.w   - cmd.clip_rect.y)) as i32,
						);
						assert!( mem::size_of::<imgui::ImDrawIdx>() == 2 );
						gl::DrawElements( gl::TRIANGLES, cmd.elem_count as i32, gl::UNSIGNED_SHORT, offset as *const c_void );
					}
					offset += cmd.elem_count as usize * mem::size_of::<imgui::ImDrawIdx>();
				}
			}

			gl::BindTexture( gl::TEXTURE_2D, 0 );
			gl::BindVertexArray( 0 );
			gl::BindBuffer( gl::ELEMENT_ARRAY_BUFFER, 0 );
			gl::BindBuffer( gl::ARRAY_BUFFER, 0 );
			gl::UseProgram( 0 );
			gl::Disable( gl::SCISSOR_TEST );
		}
	}
}
