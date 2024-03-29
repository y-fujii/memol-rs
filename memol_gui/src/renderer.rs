// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::imgui;
use std::*;

pub struct Texture {
    pub id: u32,
    pub size: (i32, i32),
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

impl Texture {
    pub fn new() -> Self {
        unsafe {
            let mut id = 0;
            gl::GenTextures(1, &mut id);
            let this = Texture { id: id, size: (0, 0) };
            this.bind();
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            this
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub unsafe fn upload_a8(&mut self, data: *const u8, w: i32, h: i32) {
        self.size = (w, h);
        self.bind();
        let swizzle = [gl::ONE as i32, gl::ONE as i32, gl::ONE as i32, gl::RED as i32];
        gl::TexParameteriv(gl::TEXTURE_2D, gl::TEXTURE_SWIZZLE_RGBA, swizzle.as_ptr());
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::R8 as i32,
            w,
            h,
            0,
            gl::RED,
            gl::UNSIGNED_BYTE,
            data as *const _,
        );
    }

    pub unsafe fn upload_u32(&mut self, data: *const u8, w: i32, h: i32) {
        self.size = (w, h);
        self.bind();
        let swizzle = [gl::RED as i32, gl::GREEN as i32, gl::BLUE as i32, gl::ALPHA as i32];
        gl::TexParameteriv(gl::TEXTURE_2D, gl::TEXTURE_SWIZZLE_RGBA, swizzle.as_ptr());
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::SRGB8_ALPHA8 as i32,
            w,
            h,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            data as *const _,
        );
    }
}

pub struct Renderer {
    program: u32,
    loc_scale: i32,
    vao: u32,
    vbo: u32,
    ebo: u32,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteBuffers(1, &self.ebo);

            let mut shaders: [u32; 2] = [0; 2];
            gl::GetAttachedShaders(
                self.program,
                shaders.len() as i32,
                ptr::null_mut(),
                shaders.as_mut_ptr(),
            );
            gl::DeleteProgram(self.program);
            for s in shaders.iter() {
                gl::DeleteShader(*s);
            }
        }
    }
}

const VERT_SHADER_CODE: &'static str = r#"
uniform vec2 u_scale;
layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_uv;
layout(location = 2) in vec4 a_color;
out vec2 m_uv;
out vec4 m_color;

void main() {
  m_uv = a_uv;
  m_color = a_color;
  gl_Position = vec4(u_scale * a_pos + vec2(-1.0, +1.0), 0.0, 1.0);
}
"#;

const FRAG_SHADER_CODE: &'static str = r#"
uniform sampler2D u_texture;
in vec2 m_uv;
in vec4 m_color;
out vec4 o_color;

void main() {
  o_color = m_color * texture(u_texture, m_uv);
}
"#;

unsafe fn compile_shader(ty: u32, code: &[&str]) -> u32 {
    let shader = gl::CreateShader(ty);
    let ptrs: Vec<_> = code.iter().map(|e| e.as_ptr() as *const _).collect();
    let lens: Vec<_> = code.iter().map(|e| e.len() as i32).collect();
    gl::ShaderSource(shader, code.len() as i32, ptrs.as_ptr(), lens.as_ptr());
    gl::CompileShader(shader);
    let mut success = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    assert!(success != 0);
    shader
}

impl Renderer {
    pub fn new(es_profile: bool) -> Self {
        unsafe {
            // shader program.
            let version = if es_profile {
                "#version 300 es\n"
            } else {
                "#version 330\n"
            };
            let vert = compile_shader(gl::VERTEX_SHADER, &[version, VERT_SHADER_CODE]);
            let frag = compile_shader(gl::FRAGMENT_SHADER, &[version, FRAG_SHADER_CODE]);
            let prog = gl::CreateProgram();
            gl::AttachShader(prog, vert);
            gl::AttachShader(prog, frag);
            gl::LinkProgram(prog);
            gl::UseProgram(prog);
            gl::Uniform1i(gl::GetUniformLocation(prog, "u_texture\0".as_ptr() as _), 0);
            let loc_scale = gl::GetUniformLocation(prog, "u_scale\0".as_ptr() as _);

            // vertex objects.
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            let mut ebo = 0;
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::EnableVertexAttribArray(2);

            let size = mem::size_of::<imgui::ImDrawVert>() as i32;
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, size, 0 as *const _);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, size, 8 as *const _);
            gl::VertexAttribPointer(2, 4, gl::UNSIGNED_BYTE, gl::TRUE, size, 16 as *const _);

            // unbind.
            gl::UseProgram(0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            Renderer {
                program: prog,
                loc_scale: loc_scale,
                vao: vao,
                vbo: vbo,
                ebo: ebo,
            }
        }
    }

    pub fn clear(&self, col: imgui::ImVec4) {
        unsafe {
            gl::ClearColor(col.x, col.y, col.z, col.w);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn sync(&self) {
        unsafe {
            // seems weird, but it is a workaround for some GPUs.
            let sync = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
            gl::Finish();
            gl::ClientWaitSync(sync, 0, 100_000_000);
            gl::DeleteSync(sync);
        }
    }

    pub fn render(&mut self, draw_data: &imgui::ImDrawData, display_size: imgui::ImVec2) {
        unsafe {
            gl::Enable(gl::FRAMEBUFFER_SRGB);
            gl::Enable(gl::BLEND);
            // XXX: premultiplied alpha is better for interpolation.
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BlendFuncSeparate(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA, gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::SCISSOR_TEST);
            gl::ActiveTexture(gl::TEXTURE0);

            gl::Viewport(0, 0, display_size.x as i32, display_size.y as i32);
            gl::UseProgram(self.program);
            gl::Uniform2f(self.loc_scale, 2.0 / display_size.x, -2.0 / display_size.y);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            gl::BindVertexArray(self.vao);

            for i in 0..draw_data.CmdListsCount {
                let cmd_list = &**draw_data.CmdLists.Data.offset(i as isize);

                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    cmd_list.VtxBuffer.Size as isize * mem::size_of::<imgui::ImDrawVert>() as isize,
                    cmd_list.VtxBuffer.Data as *const _,
                    gl::STREAM_DRAW,
                );
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    cmd_list.IdxBuffer.Size as isize * mem::size_of::<imgui::ImDrawIdx>() as isize,
                    cmd_list.IdxBuffer.Data as *const _,
                    gl::STREAM_DRAW,
                );

                let mut offset = 0;
                for i in 0..cmd_list.CmdBuffer.Size {
                    let cmd = &*cmd_list.CmdBuffer.Data.offset(i as isize);
                    if let Some(cb) = cmd.UserCallback {
                        cb(cmd_list, cmd);
                    } else {
                        gl::BindTexture(gl::TEXTURE_2D, cmd.TextureId as u32);
                        gl::Scissor(
                            cmd.ClipRect.x as i32,
                            (display_size.y - cmd.ClipRect.w) as i32,
                            (cmd.ClipRect.z - cmd.ClipRect.x) as i32,
                            (cmd.ClipRect.w - cmd.ClipRect.y) as i32,
                        );
                        debug_assert!(mem::size_of::<imgui::ImDrawIdx>() == 2);
                        gl::DrawElements(
                            gl::TRIANGLES,
                            cmd.ElemCount as i32,
                            gl::UNSIGNED_SHORT,
                            offset as *const _,
                        );
                    }
                    offset += cmd.ElemCount as usize * mem::size_of::<imgui::ImDrawIdx>();
                }
            }

            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::UseProgram(0);
            gl::Disable(gl::SCISSOR_TEST);
        }
    }
}

pub fn update_font_texture(font: &mut imgui::ImFontAtlas) {
    destroy_font_texture(font);

    let mut tex = Texture::new();
    let mut data = ptr::null_mut();
    let mut w = 0;
    let mut h = 0;
    unsafe {
        font.GetTexDataAsAlpha8(&mut data, &mut w, &mut h, ptr::null_mut());
        tex.upload_a8(data, w, h);
    }
    font.TexID = tex.id as *mut _;
    mem::forget(tex);
}

pub fn destroy_font_texture(font: &mut imgui::ImFontAtlas) {
    let id = mem::replace(&mut font.TexID, ptr::null_mut());
    if !id.is_null() {
        unsafe { gl::DeleteTextures(1, &(id as u32)) };
    }
}
