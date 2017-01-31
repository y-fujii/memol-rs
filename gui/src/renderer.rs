// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::*;
use std::os::raw::{ c_void, c_char };
use gl;
use imgui;


pub struct Renderer {
	pub font_texture: u32,
	pub program: u32,
	pub loc_scale: i32,
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

			(*imgui::get_io().Fonts).TexID = ptr::null_mut();
			imgui::Shutdown();
		}
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
		out_color = vec4( frag_color.xyz, frag_color.w * texture( Texture, frag_uv ).x );
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
		unsafe {
			let io = imgui::get_io();

			let mut data = ptr::null_mut();
			let mut w = 0;
			let mut h = 0;
			(*io.Fonts).GetTexDataAsAlpha8( &mut data, &mut w, &mut h, ptr::null_mut() );

			// font texture.
			let mut tex = 0;
			gl::GenTextures( 1, &mut tex );
			gl::BindTexture( gl::TEXTURE_2D, tex );
			gl::TexParameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32 );
			gl::TexParameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );
			gl::TexImage2D( gl::TEXTURE_2D, 0, gl::R8 as i32, w, h, 0, gl::RED, gl::UNSIGNED_BYTE, data as *const c_void );
			(*io.Fonts).TexID = mem::transmute( tex as usize );

			// shader program.
			let vert = compile_shader( gl::VERTEX_SHADER,   VERT_SHADER_CODE );
			let frag = compile_shader( gl::FRAGMENT_SHADER, FRAG_SHADER_CODE );
			let prog = gl::CreateProgram();
			gl::AttachShader( prog, vert );
			gl::AttachShader( prog, frag );
			gl::LinkProgram( prog );

			gl::Uniform1i( gl::GetUniformLocation( prog, "Texture\0".as_ptr() as *const c_char ), 0 );
			let loc_scale = gl::GetUniformLocation( prog, "Scale\0".as_ptr() as *const c_char );

			// vertex objects.
			let mut vao = 0;
			gl::GenVertexArrays( 1, &mut vao );
			gl::BindVertexArray( vao );

			let mut vbo = 0;
			gl::GenBuffers( 1, &mut vbo );
			gl::BindBuffer( gl::ARRAY_BUFFER, vbo );

			let mut ebo = 0;
			gl::GenBuffers( 1, &mut ebo );
			gl::BindBuffer( gl::ELEMENT_ARRAY_BUFFER, ebo );

			gl::EnableVertexAttribArray( 0 );
			gl::EnableVertexAttribArray( 1 );
			gl::EnableVertexAttribArray( 2 );

			let size = mem::size_of::<imgui::ImDrawVert>() as i32;
			gl::VertexAttribPointer( 0, 2, gl::FLOAT,         gl::FALSE, size,  0 as *const c_void );
			gl::VertexAttribPointer( 1, 2, gl::FLOAT,         gl::FALSE, size,  8 as *const c_void );
			gl::VertexAttribPointer( 2, 4, gl::UNSIGNED_BYTE, gl::TRUE,  size, 16 as *const c_void );

			// unbind.
			gl::BindTexture( gl::TEXTURE_2D, 0 );
			gl::BindVertexArray( 0 );
			gl::BindBuffer( gl::ELEMENT_ARRAY_BUFFER, 0 );
			gl::BindBuffer( gl::ARRAY_BUFFER, 0 );

			Renderer {
				font_texture: tex,
				program: prog,
				loc_scale: loc_scale,
				vao: vao,
				vbo: vbo,
				ebo: ebo,
			}
		}
	}

	pub fn render( &self ) {
		unsafe {
			let io = imgui::get_io();
			let draw_data = imgui::get_draw_data();

			gl::Enable( gl::BLEND );
			gl::BlendEquation( gl::FUNC_ADD );
			gl::BlendFunc( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA );
			gl::Disable( gl::CULL_FACE );
			gl::Disable( gl::DEPTH_TEST );
			gl::Enable( gl::SCISSOR_TEST );
			gl::ActiveTexture( gl::TEXTURE0 );

			gl::Viewport( 0, 0,
				(io.DisplaySize.x * io.DisplayFramebufferScale.x) as i32,
				(io.DisplaySize.y * io.DisplayFramebufferScale.y) as i32,
			);
			gl::UseProgram( self.program );
			gl::Uniform2f( self.loc_scale, 2.0 / io.DisplaySize.x, -2.0 / io.DisplaySize.y );
			gl::BindVertexArray( self.vao );

			let vtx_size = (0 .. draw_data.CmdListsCount as isize)
				.map( |i| (**draw_data.CmdLists.offset( i )).VtxBuffer.Size )
				.max()
				.unwrap_or( 0 );
			gl::BindBuffer( gl::ARRAY_BUFFER, self.vbo );
			gl::BufferData(
				gl::ARRAY_BUFFER,
				vtx_size as isize * mem::size_of::<imgui::ImDrawVert>() as isize,
				ptr::null(),
				gl::STREAM_DRAW,
			);

			let idx_size = (0 .. draw_data.CmdListsCount as isize)
				.map( |i| (**draw_data.CmdLists.offset( i )).IdxBuffer.Size )
				.max()
				.unwrap_or( 0 );
			gl::BindBuffer( gl::ELEMENT_ARRAY_BUFFER, self.ebo );
			gl::BufferData(
				gl::ELEMENT_ARRAY_BUFFER,
				idx_size as isize * mem::size_of::<imgui::ImDrawIdx>() as isize,
				ptr::null(),
				gl::STREAM_DRAW,
			);

			for i in 0 .. draw_data.CmdListsCount {
				let cmd_list = &**draw_data.CmdLists.offset( i as isize );

				gl::BindBuffer( gl::ARRAY_BUFFER, self.vbo );
				gl::BufferSubData(
					gl::ARRAY_BUFFER,
					0,
					cmd_list.VtxBuffer.Size as isize * mem::size_of::<imgui::ImDrawVert>() as isize,
					cmd_list.VtxBuffer.Data as *const c_void,
				);

				gl::BindBuffer( gl::ELEMENT_ARRAY_BUFFER, self.ebo );
				gl::BufferSubData(
					gl::ELEMENT_ARRAY_BUFFER,
					0,
					cmd_list.IdxBuffer.Size as isize * mem::size_of::<imgui::ImDrawIdx>() as isize,
					cmd_list.IdxBuffer.Data as *const c_void,
				);

				let mut offset = 0;
				for i in 0 .. cmd_list.CmdBuffer.Size {
					let cmd = &*cmd_list.CmdBuffer.Data.offset( i as isize );
					if let Some( cb ) = cmd.UserCallback {
						cb( cmd_list, cmd );
					}
					else {
						gl::BindTexture( gl::TEXTURE_2D, cmd.TextureId as u32 );
						gl::Scissor(
							(io.DisplayFramebufferScale.x * cmd.ClipRect.x) as i32,
							(io.DisplayFramebufferScale.y * (io.DisplaySize.y - cmd.ClipRect.w)) as i32,
							(io.DisplayFramebufferScale.x * (cmd.ClipRect.z   - cmd.ClipRect.x)) as i32,
							(io.DisplayFramebufferScale.y * (cmd.ClipRect.w   - cmd.ClipRect.y)) as i32,
						);
						assert!( mem::size_of::<imgui::ImDrawIdx>() == 2 );
						gl::DrawElements( gl::TRIANGLES, cmd.ElemCount as i32, gl::UNSIGNED_SHORT, offset as *const c_void );
					}
					offset += cmd.ElemCount as usize * mem::size_of::<imgui::ImDrawIdx>();
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
