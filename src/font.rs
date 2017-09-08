extern crate glfw;

use std;
use gl;
use gl::types::*;
use glutil;

// Shader sources
static VS_SRC: &'static str = include_str!("fontvs.glsl");
static FS_SRC: &'static str = include_str!("fontfs.glsl");


pub struct Font {
    program : GLuint,
    zoom_factor : f32,
}

impl Font {
    pub fn new() -> Font {
        let mut texture : GLuint = 0;
        let bytes = include_bytes!("fonts/Osborne_I.charrom");
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);            
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R8UI as GLint,
                           1024, 10, 0,
                           gl::RED_INTEGER,gl::UNSIGNED_BYTE, 
                           bytes.as_ptr() as *const _);
        }

        let program = glutil::build_program(VS_SRC, FS_SRC)
            .expect("Failed to create font shader program");
        Font {
            program : program,
            zoom_factor : 2.0,
        }
    }

    pub fn width(&self,text:&str) -> u32 { (self.zoom_factor*8.0*text.len() as f32) as u32 }
    pub fn height(&self) -> u32 { (self.zoom_factor*10.0) as u32 }

    pub fn draw(&self, 
                window_size : (u32, u32),
                text_pos : (u32, u32),
                text : &str) {
        let charw = 1.0 / 128.0;
        // chars are 8x10
        let pw = 2.0 / window_size.0 as f32;
        let ph = 2.0 / window_size.1 as f32;
        let cw = self.zoom_factor * 8.0 * pw;
        let ch = self.zoom_factor * 10.0 * ph;
        let mut x = -1.0 + (text_pos.0 as f32 * pw);
        let y = (1.0 - ch) - (text_pos.1 as f32 * ph);

        const INDICES : [u16; 6] = [
            0, 1, 2,
            0, 3, 2,
        ];
        let mut vbo : GLuint = 0;
        let mut ebo : GLuint = 0;
        let mut vao : GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1,&mut vao);
            gl::BindVertexArray(vao);
            gl::GenBuffers(1,&mut vbo);
            gl::GenBuffers(1,&mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, 4*6, INDICES.as_ptr() as *const _, gl::STATIC_DRAW);
            gl::UseProgram(self.program);
            gl::BindFragDataLocation(self.program, 0,
                                     std::ffi::CString::new("color").unwrap().as_ptr());
            let pos_attrib = glutil::attrib_loc(self.program,"position") as GLuint;
            gl::EnableVertexAttribArray(pos_attrib);
            gl::VertexAttribPointer(pos_attrib, 2, gl::FLOAT, gl::FALSE,
                                    4*4, std::ptr::null());
            let tex_attrib = glutil::attrib_loc(self.program,"tex_coords") as GLuint;
            gl::EnableVertexAttribArray(tex_attrib as GLuint);
            gl::VertexAttribPointer(tex_attrib, 2, gl::FLOAT, gl::FALSE,
                                    4*4, (2*4) as *const _);
        }

        for b in text.as_bytes() {
            let tex_left = charw * *b as f32;

            // Two triangles to cover the character
            let vertices : [GLfloat; 16] = [
                x,y+ch,    tex_left,0.0,
                x+cw,y+ch, tex_left+charw,0.0,
                x+cw,y,   tex_left+charw,1.0,
                x,y,       tex_left,1.0,
            ];
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                gl::BufferData(gl::ARRAY_BUFFER, 4*16, vertices.as_ptr() as *const _, gl::STATIC_DRAW);
                gl::Uniform4f(glutil::uniloc(self.program,"in_color"), 0.0,1.0,0.0,0.9);
                gl::Uniform1i(glutil::uniloc(self.program,"tex"), 2);
                gl::BindVertexArray(vao);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
            }
            x = x + cw;
        }
        
    }
}

