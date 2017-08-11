extern crate glfw;
extern crate gl;

use glfw::{Action, Context, Key};
use gl::types::*;
use std;
use std::mem;
use std::ptr;
use std::str;
use std::ffi::CString;

// Two triangles to cover the window
static VERTEX_DATA: [GLfloat; 12] = [
    -1.0, 1.0, -1.0, -1.0, 1.0, -1.0,
    -1.0, 1.0, 1.0, 1.0, 1.0, -1.0,
];

// Shader sources
static VS_SRC: &'static str = include_str!("vs.glsl");

static FS_SRC: &'static str = include_str!("fs.glsl");

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(shader,
                                 len,
                                 ptr::null_mut(),
                                 buf.as_mut_ptr() as *mut GLchar);
            panic!("{}",
                   str::from_utf8(&buf)
                       .ok()
                       .expect("ShaderInfoLog not valid utf8"));
        }
    }
    shader
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(program,
                                  len,
                                  ptr::null_mut(),
                                  buf.as_mut_ptr() as *mut GLchar);
            panic!("{}",
                   str::from_utf8(&buf)
                       .ok()
                       .expect("ProgramInfoLog not valid utf8"));
        }
        program
    }
}

pub struct Visualizer {
    pub glfw : glfw::Glfw,
    pub win : glfw::Window,
    events : std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,
    program : GLuint,
    data : Vec<u8>,
    data_len : usize,
    stride : u32,
    zoom : f32,
}

impl Visualizer {
    pub fn new(size : (u32, u32), data_sz : usize) -> Visualizer {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        let (mut window, events) = glfw.create_window(size.0, size.1 ,
                                                      "ROM explorer",
                                                      glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");
        window.set_key_polling(true);
        window.make_current();
        gl::load_with(|name| window.get_proc_address(name) as *const _);
        let mut mts : i32 = 0;
        unsafe {
        gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut mts as *mut i32 );
        }
        println!("Max texture size: {}", mts);
        // Create vertex shader
        let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
        // Fragment shader needs size of data at compile time (in uints)
        let fsstr = String::from(FS_SRC).replace("{}",(data_sz).to_string().as_str());
        let fs = compile_shader(fsstr.as_str(), gl::FRAGMENT_SHADER);
        let program = link_program(vs, fs);

        let mut vao = 0; let mut vbo = 0;
        unsafe {
        // Create Vertex Array Object
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            // Create a Vertex Buffer Object and copy the vertex data to it
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER,
                           (VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                           mem::transmute(&VERTEX_DATA[0]),
                           gl::STATIC_DRAW);
            // Use shader program
            gl::UseProgram(program);
            gl::BindFragDataLocation(program, 0, CString::new("out_color").unwrap().as_ptr());
            // Specify the layout of the vertex data
            let pos_attr = gl::GetAttribLocation(program, CString::new("position").unwrap().as_ptr());
            gl::EnableVertexAttribArray(pos_attr as GLuint);
            gl::VertexAttribPointer(pos_attr as GLuint,
                                    2,
                                    gl::FLOAT,
                                    gl::FALSE as GLboolean,
                                    0,
                                    ptr::null());
        }
        let vz = Visualizer {
            glfw : glfw,
            win : window,
            events: events,
            program : program,
            data : Vec::new(),
            data_len : 0,
            stride : 8,
            zoom : 1.0,
        };
        vz.set_size(size);
        vz
    }

    fn uniform_loc(&self, location : &str) -> GLint {
        unsafe {
            gl::GetUniformLocation(self.program, CString::new(location).unwrap().as_ptr())
        }
    }
    
    pub fn set_data(&mut self, dat : &[u8]) {
        unsafe {
            // Load image as texture
            let mut texo = 0;
            gl::GenTextures(1, &mut texo);
            gl::BindTexture(gl::TEXTURE_2D, texo);
            let maxw : usize = 16384;
            let tw : usize = maxw;
            let th : usize = (dat.len() + (maxw-1))/maxw;
            self.data_len = dat.len();
            self.data.reserve(tw*th);
            self.data.extend(dat.iter().cloned());
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R8UI as i32, tw as GLsizei, th as GLsizei, 0,
                gl::RED_INTEGER, gl::UNSIGNED_BYTE, self.data.as_ptr() as *const GLvoid);
            println!("Texture bound at {}, {}",texo, dat.len());
            gl::Uniform1ui(self.uniform_loc("romh"),th as u32);
            gl::Uniform1i(self.uniform_loc("romtex"), 0);
            gl::BindTexture(gl::TEXTURE_1D, 0 );
        }
    }

    pub fn set_selection(&self, start : u32, finish : u32) {
        unsafe {
            gl::Uniform1ui(self.uniform_loc("sel0"),start);
            gl::Uniform1ui(self.uniform_loc("sel1"),finish);
        }
    }
    
    pub fn set_zoom(&mut self, zoom : f32) {
        self.zoom = zoom;
        unsafe {
            gl::Uniform1f(self.uniform_loc("zoom"),1.0/zoom);
        }
    }

    pub fn set_size(&self, size : (u32, u32)) {
        unsafe {
            gl::Uniform1ui(self.uniform_loc("ww"),size.0);
            gl::Uniform1ui(self.uniform_loc("wh"),size.1);
        }
    }

    pub fn set_stride(&mut self, stride : u32) {
        self.stride = stride;
        unsafe {
            gl::Uniform1ui(self.uniform_loc("stride"),stride);
        }
    }

    pub fn render(&mut self) {
        unsafe { 
            gl::ClearColor(1.0,0.0,0.0,1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::BindTexture(gl::TEXTURE_1D, 1 );
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
        self.win.swap_buffers();
    }

    pub fn handle_events(&mut self) {
        let events = glfw::flush_messages(&self.events).map(|(_,e)| e).collect::<Vec<glfw::WindowEvent>>();
        for event in events {
            match event {
                glfw::WindowEvent::Key(Key::Right, _, Action::Press, _) |
                glfw::WindowEvent::Key(Key::Right, _, Action::Repeat, _) => {
                    if self.stride < (self.data_len - 7) as u32 {
                        let s = self.stride + 8;
                        self.set_stride(s);
                    }
                },
                glfw::WindowEvent::Key(Key::Left, _, Action::Press, _) |
                glfw::WindowEvent::Key(Key::Left, _, Action::Repeat, _)=> {
                    if self.stride > 8 {
                        let s = self.stride - 8;
                        self.set_stride(s);
                    }
                },
                glfw::WindowEvent::Key(Key::Down, _, Action::Press, _) => {
                    let z = if self.zoom <= 1.0 { self.zoom / 2.0 } else { self.zoom - 1.0 };
                    self.set_zoom(z);
                },
                glfw::WindowEvent::Key(Key::Up, _, Action::Press, _) => {
                    let z = if self.zoom >= 1.0 { self.zoom + 1.0 } else { self.zoom * 2.0 };
                    self.set_zoom(z);
                },
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.win.set_should_close(true)
                },
                _ => {},
            }
        }
    }
}
