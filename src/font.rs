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
    vbo : GLuint,
    vao : GLuint,
}


const INDICES : [GLuint; 6] = [
    0, 1, 2,
    0, 3, 2,
];

impl Font {
    pub fn new() -> Font {
        let mut texture : GLuint = 0;
        let bytes = include_bytes!("fonts/Osborne_I.charrom");
        let mut vbo : GLuint = 0;
        let mut ebo : GLuint = 0;
        let mut vao : GLuint = 0;

        let program = glutil::build_program(VS_SRC, FS_SRC)
            .expect("Failed to create font shader program");

        unsafe {
            gl::GenVertexArrays(1,&mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(1,&mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::GenBuffers(1,&mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, 4*6, INDICES.as_ptr() as *const _, gl::STATIC_DRAW);

            gl::UseProgram(program);
            gl::BindFragDataLocation(program, 0,
                                     std::ffi::CString::new("color").unwrap().as_ptr());
            let pos_attrib = glutil::attrib_loc(program,"position");
            gl::EnableVertexAttribArray(pos_attrib as GLuint);
            gl::VertexAttribPointer(pos_attrib as GLuint, 2, gl::FLOAT, gl::FALSE,
                                    4*4, std::ptr::null());
            let tex_attrib = glutil::attrib_loc(program,"tex_coords");
            gl::EnableVertexAttribArray(tex_attrib as GLuint);
            gl::VertexAttribPointer(tex_attrib as GLuint, 2, gl::FLOAT, gl::FALSE,
                                    4*4, (2*4) as *const _);
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
        Font {
            program : program,
            zoom_factor : 2.0,
            vbo : vbo,
            vao : vao,
        }
    }

    pub fn size(&self,text:&str) -> (f32, f32) {
        let mut w : f32 = 0.0;
        let mut maxw : f32 = 0.0;
        let cw = 8.0 * self.zoom_factor;
        let ch = 10.0 * self.zoom_factor;
        let mut h = ch;
        for c in text.bytes() {
            match c {
                b'\r' => {},
                b'\0' => {},
                b'\n' => { maxw = maxw.max(w); w = 0.0; h = h + ch; }
                _ => { w = w + cw; }
            }
        }
        (maxw.max(w),h)
    }

    pub fn width(&self,text:&str) -> i32 { self.size(text).0 as i32 }
    pub fn height(&self,text:&str) -> i32 { self.size(text).1 as i32 }

    pub fn draw(&self, 
                window_size : (i32, i32),
                text_pos : (i32, i32),
                text : &str) {
        let charw = 1.0 / 128.0;
        // chars are 8x10
        let pw = 2.0 / window_size.0 as f32;
        let ph = 2.0 / window_size.1 as f32;
        let cw = self.zoom_factor * 8.0 * pw;
        let ch = self.zoom_factor * 10.0 * ph;
        let mut x = -1.0 + (text_pos.0 as f32 * pw);
        let mut y = (1.0 - ch) - (text_pos.1 as f32 * ph);

        unsafe {
            gl::UseProgram(self.program);
            gl::Uniform4f(glutil::uniloc(self.program,"in_color"), 0.0,1.0,0.0,0.9);
            gl::Uniform1i(glutil::uniloc(self.program,"tex"), 2);
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        }
        for b in text.as_bytes() {
            match *b {
                0x0D => { continue; },
                0x0A => { 
                    x = -1.0 + (text_pos.0 as f32 * pw);
                    y = y - ch;
                    continue;
                },
                _ => {}
            }
            let tex_left = charw * *b as f32;

            // Two triangles to cover the character
            let vertices : [GLfloat; 16] = [
                x,y+ch,    tex_left,0.0,
                x+cw,y+ch, tex_left+charw,0.0,
                x+cw,y,   tex_left+charw,1.0,
                x,y,       tex_left,1.0,
            ];
            //println!("x1 {} y1 {} x2 {} y2 {}",vertices[0], vertices[13],vertices[4],vertices[5]);
            unsafe {
                gl::BufferData(gl::ARRAY_BUFFER, 4*16, vertices.as_ptr() as *const _, gl::STATIC_DRAW);
                gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
            }
            x = x + cw;
        }
        
    }
}

