use gl;
use gl::types::*;
use glutil;
use std;

// Shader sources
static VS_SRC: &'static str = include_str!("panelvs.glsl");
static FS_SRC: &'static str = include_str!("panelfs.glsl");

pub struct Panel {
	panel_width_px : u32,
    vbo : GLuint,
    vao : GLuint,
    size : (i32, i32),
    char_cell_px : (i32, i32),
    char_dim : (i32, i32),
    contents : Vec<u8>,
    contents_tex : GLuint,
    program : GLuint,
}

const INDICES : [GLuint; 6] = [
    0, 1, 2,
    0, 3, 2,
];

impl Panel {
	pub fn new(panel_width_px : u32, size : (i32, i32) ) -> Panel {
        let mut texture : GLuint = 0;
        let mut contents_tex : GLuint = 0;
        let bytes = include_bytes!("fonts/Osborne_I.charrom");
        let mut vbo : GLuint = 0;
        let mut ebo : GLuint = 0;
        let mut vao : GLuint = 0;

        let program = glutil::build_program(VS_SRC, FS_SRC)
            .expect("Failed to create panel shader program");
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
            gl::GenTextures(1, &mut contents_tex);
            gl::ActiveTexture(gl::TEXTURE3);
            gl::BindTexture(gl::TEXTURE_2D, contents_tex);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);            
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
        }

		let mut p = Panel {
			panel_width_px : panel_width_px,
			vbo : vbo,
			vao : vao,
			char_cell_px : (8,10),
			char_dim : (0,0),
			size : (0,0),
			contents : Vec::new(),
			contents_tex : contents_tex,
			program : program,
		};
		p.update_size(size);
		p
	}

	fn update_size(&mut self, size : (i32, i32)) {
		if self.size == size { return }
		self.size = size;
		// compute n, m
		fn compute_n_u0_u1(px_sz : i32, cell_sz : i32) -> (i32, GLfloat, GLfloat) {
			let n  = (px_sz-2)/cell_sz;
			let off = (px_sz - n*cell_sz)/2;
			let u0 = -off as GLfloat/cell_sz as GLfloat;
			let off2 = px_sz - (off+n*cell_sz);
			let u1 = n as GLfloat + off2 as GLfloat/cell_sz as GLfloat;
			(n,u0,u1)
		}
		let (n, u0, u1) = compute_n_u0_u1(self.panel_width_px as i32, self.char_cell_px.0);
		let (m, v0, v1) = compute_n_u0_u1(size.1, self.char_cell_px.1);


		let x0 : GLfloat = 1.0-(2.0*self.panel_width_px as GLfloat/size.0 as GLfloat);

        let vertices : [GLfloat; 16] = [
        	x0, 1.0,    u0, v0,
        	1.0, 1.0,   u1, v0, 
        	1.0, -1.0,  u1, v1,
        	x0, -1.0,   u0, v1,
        ];
        //println!("x1 {} y1 {} x2 {} y2 {}",u0,v0,u1,v1);
        self.contents.resize( (n*m) as usize, 32);
        unsafe {
            gl::UseProgram(self.program);
            gl::Uniform4f(glutil::uniloc(self.program,"bounds"), 0.0,0.0,n as GLfloat,m as GLfloat);
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        	gl::BufferData(gl::ARRAY_BUFFER, 4*16, vertices.as_ptr() as *const _, gl::STATIC_DRAW);
        }
        self.char_dim = (n, m);
	}

	pub fn render(&mut self, size : (i32, i32)) {
		self.update_size(size);
		unsafe {
            gl::UseProgram(self.program);
            gl::Uniform4f(glutil::uniloc(self.program,"in_color"), 0.0,1.0,0.0,0.9);
            gl::Uniform1i(glutil::uniloc(self.program,"tex"), 2);
            gl::Uniform1i(glutil::uniloc(self.program,"characters"), 3);
            gl::BindVertexArray(self.vao);
            gl::ActiveTexture(gl::TEXTURE3);
            gl::BindTexture(gl::TEXTURE_2D, self.contents_tex);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R8UI as GLint,
                           self.char_dim.0, self.char_dim.1, 0,
                           gl::RED_INTEGER,gl::UNSIGNED_BYTE, 
                           self.contents.as_ptr() as *const _);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
        }
	}
}