extern crate glfw;

use glfw::{Window,WindowEvent};
use gl;
use gl::types::*;

use std;
use std::str;

use annotation;

// Shader sources
static VS_SRC: &'static str = include_str!("vs.glsl");

static FS_SRC: &'static str = include_str!("fs.glsl");

use std::collections::HashSet;

pub struct MouseState {
    start_drag_pos : Option<(f64,f64)>,
    last_pos : (f64, f64),
//    down : HashSet<glutin::MouseButton>,
    start_ul_offset : (f32, f32),
}

impl MouseState {
    pub fn new() -> MouseState {
        MouseState { 
            start_drag_pos : None, 
            last_pos : (0.0, 0.0),
//            down : HashSet::new(),
            start_ul_offset : (0.0, 0.0),
        }
    }
}

pub struct Visualizer<'a> {
    pub window : glfw::Window,
    pub events : std::sync::mpsc::Receiver<(f64, WindowEvent)>,
    program : GLuint,

    positions : GLuint,
    indices : GLuint,
    data_len : usize,
    /// width, in bits, of each column
    stride : u32,
    /// height, in rows, of each colum
    col_height : u32,
    /// start and end of current selection, as byte idx
    selection : (u32, u32),
    texture : GLuint,
    annotation_tex : GLuint,
    annotation_d : Vec<u8>,
    zoom : f32,
    ul_offset : (f32, f32), // offset of upper left hand corner IN PX OF CURRENT ZOOM
    pub closed : bool,
    mouse_state : MouseState,
    dat : &'a [u8],
    annotation_store : Option<annotation::AnnotationStore>,
}

fn build_shader(src : &str, shader_type : GLenum) -> Option<GLuint> {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        let src_cstr = std::ffi::CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &src_cstr.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);
        let mut compiled : GLint = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compiled);
        if compiled == gl::TRUE as GLint {
            Some(shader)
        } else {
            gl::DeleteShader(shader);
            None
        }
    }
}

fn build_program(vertex_shader_src : &str, fragment_shader_src : &str) -> Option<GLuint> {
    unsafe {
        let program = gl::CreateProgram();
        match (build_shader(vertex_shader_src, gl::VERTEX_SHADER),
               build_shader(fragment_shader_src, gl::FRAGMENT_SHADER)) {
            (Some(vs), Some(fs)) => {
                gl::AttachShader(program, vs);
                gl::AttachShader(program, fs);
                gl::LinkProgram(program);
                gl::DeleteShader(vs);
                gl::DeleteShader(fs);
                let mut linked : GLint = 0;
                gl::GetProgramiv(program, gl::LINK_STATUS, &mut linked);
                if linked == gl::TRUE as GLint {
                    Some(program)
                } else {
                    gl::DeleteProgram(program);
                    None
                }
            },
            _ => {
                gl::DeleteProgram(program);
                None
            }
        }
    }
}

const VERTICES : [GLfloat; 16] = [
    -1.0,  1.0,    0.0, 1.0,
    1.0,   1.0,    1.0, 1.0,
    1.0,  -1.0,    1.0, 0.0,
    -1.0, -1.0,    0.0, 0.0,
];

const INDICES : [u16; 6] = [
    0, 1, 2,
    0, 3, 2,
];

use std::os::raw::c_void;

impl<'a> Visualizer<'a> {
        
    pub fn new(glfw : &mut glfw::Glfw, size : (u32, u32), dat : &'a [u8]) -> Visualizer<'a> {
        let (mut window, events) = glfw.create_window(size.0, size.1,
                                                      "ROM Explorer",
                                                      glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");
        gl::load_with(|s| window.get_proc_address(s) as *const _);
        let program = build_program(VS_SRC, FS_SRC).unwrap();

        let mut positions : GLuint = 0;
        unsafe {
            gl::GenBuffers(1,&mut positions);
            gl::BindBuffer(gl::ARRAY_BUFFER, positions);
            gl::BufferData(gl::ARRAY_BUFFER, 4*16, VERTICES.as_ptr() as *const c_void, gl::STATIC_DRAW);
        }

        let mut indices : GLuint = 0;
        unsafe {
            gl::GenBuffers(1,&mut indices);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, indices);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, 4*6, INDICES.as_ptr() as *const c_void, gl::STATIC_DRAW);
            gl::UseProgram(program);
        }

        fn get_attrib_location(program : GLuint , name : &str) -> GLint {
            let c_str = std::ffi::CString::new(name.as_bytes()).unwrap();
            unsafe { gl::GetAttribLocation(program, c_str.as_ptr()) }
        }
        let pos_attrib = get_attrib_location(program,"position") as GLuint;
        unsafe {
            gl::EnableVertexAttribArray(pos_attrib);
            gl::VertexAttribPointer(pos_attrib, 2, gl::FLOAT, gl::FALSE,
                                    4*4, std::ptr::null());
            let tex_attrib = get_attrib_location(program,"tex_coords") as GLuint;
            gl::EnableVertexAttribArray(tex_attrib as GLuint);
            gl::VertexAttribPointer(tex_attrib, 2, gl::FLOAT, gl::FALSE,
                                    4*4, (2*4) as *const c_void);
        }
        let maxw : usize = 16384;
        let tw : usize = maxw;
        let th : usize = (dat.len() + (maxw-1))/maxw;
        let mut d : Vec<u8> = Vec::new();
        d.reserve(tw*th);
        d.extend(dat.iter().cloned());
        d.resize(tw*th,0);

        let mut texture : GLuint = 0;
        let mut annotation_tex : GLuint = 0;
        let mut annotation_d : Vec<u8> = Vec::new();
        annotation_d.resize(tw*th,0);
        let cloned_annot = annotation_d.clone();
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R8UI as GLint,
                           tw as GLsizei, th as GLsizei, 0,
                           gl::RED_INTEGER,gl::UNSIGNED_BYTE, d.as_ptr() as *const c_void);
            gl::GenTextures(1, &mut annotation_tex);
            gl::BindTexture(gl::TEXTURE_2D, annotation_tex);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R8UI as GLint,
                           tw as GLsizei, th as GLsizei, 0,
                           gl::RED_INTEGER,gl::UNSIGNED_BYTE, cloned_annot.as_ptr() as *const c_void);
        }
        
        let mut vz = Visualizer {
            window : window,
            events : events,
            program : program,
            positions : positions,
            indices : indices,
            data_len : dat.len(),
            stride : 8,
            col_height : 512,
            selection : (0,0),
            texture : texture,
            annotation_tex : annotation_tex,
            annotation_d : annotation_d,
            zoom : 1.0,
            ul_offset : (0.0, 0.0),
            closed: false,
            mouse_state : MouseState::new(),
            dat : dat,
            annotation_store : None,
        };
        vz
    }
    
    pub fn set_selection(&mut self, start : u32, finish : u32) {
        self.selection = (start, finish);
    }

    pub fn set_stride(&mut self, stride : u32) {
        self.stride = stride;
    }

    pub fn uniloc(&self, name : &str) -> GLint {
        let c_str = std::ffi::CString::new(name.as_bytes()).unwrap();
        unsafe { gl::GetUniformLocation(self.program, c_str.as_ptr()) }
    }
    
    pub fn render(&mut self) {
        let size = self.window.get_size();
        unsafe {
            gl::UseProgram(self.program);
            gl::ClearColor(1.0,0.0,0.0,1.0);
            gl::Uniform4ui(self.uniloc("win"),0,0,size.0 as u32,size.1 as u32);
            gl::Uniform1ui(self.uniloc("bitstride"), self.stride);
            gl::Uniform1ui(self.uniloc("colstride"), self.stride*self.col_height);
            gl::Uniform1ui(self.uniloc("datalen"), self.data_len as u32);
            gl::Uniform2ui(self.uniloc("selection"), self.selection.0, self.selection.1);
            gl::Uniform1ui(self.uniloc("texwidth"), 16384 as u32);
            gl::Uniform1i(self.uniloc("romtex"), 0);
            gl::Uniform1i(self.uniloc("annotex"), 1);
            gl::Uniform2i(self.uniloc("ul_offset"), self.ul_offset.0 as i32, self.ul_offset.1 as i32);
            gl::Uniform1f(self.uniloc("zoom"),self.zoom);
                          
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
        }        
            
        let bfc = self.byte_from_coords(self.mouse_state.last_pos);
        let text = match bfc {
            Some(x) => format!("0x{:x}",x),
            None => String::new(),
        };
        //let location = (size.0 - self.font.width(text.as_str()),
        //                size.1 - self.font.height());
        //self.font.draw(&self.display, &mut target, size, location, text.as_str());
        // match bfc {
        //     Some(x) => match self.annotation_store {
        //         Some(ref store) => {
        //             let annos = store.query(x as usize);
        //             let y = 0;
        //             for a in annos {
        //                 let s = a.comments();
        //                 let location = (size.0.saturating_sub(self.font.width(s)), y);
        //                 self.font.draw(&self.display, &mut target, size, location, s);
        //             }
        //         },
        //         None => {}
        //     },
        //     None => {}
        // }
    }

    fn zoom_to_center(&mut self, cursor : (f64, f64), z : f32) {
        fn findul(ul : f32, cursor : f32, oldz : f32, newz : f32) -> f32 {
            // convert all coords to zoom level 1.0
            let ul1 = ul / oldz;
            let cursor1 = cursor / oldz;
            let ulnew1 = ul1 + (cursor1 * (1.0 - 1.0/(newz/oldz)));
            ulnew1 * newz
        }
        self.ul_offset = ( findul(self.ul_offset.0, cursor.0 as f32, self.zoom, z),
                          findul(self.ul_offset.1, cursor.1 as f32, self.zoom, z) );
        self.zoom = z;
    }

    fn zoom_to(&mut self, z : f32) {
        fn findul(ul : f32, win : u32, oldz : f32, newz : f32) -> f32 {
            let half = win as f32 / 2.0;
            let c = ul + (half / oldz);
            c - half / newz
        }
        let size = self.window.get_size();
        self.ul_offset = ( findul(self.ul_offset.0, size.0 as u32, self.zoom, z),
                          findul(self.ul_offset.1, size.1 as u32, self.zoom, z) );
        self.zoom = z;
    }
    
    fn zoom_in(&mut self) {
        let z = if self.zoom >= 1.0 { self.zoom + 0.1 } else {   1.0 };
        self.zoom_to(z);
    }

    fn zoom_out(&mut self) {
        let z = if self.zoom > 1.0 { self.zoom - 0.1 } else { 1.0 };
        self.zoom_to(z);
    }

    // fn handle_mouse_scroll(&mut self, d : glutin::MouseScrollDelta) {
    //     match d {
    //         glutin::MouseScrollDelta::LineDelta(_,v) => {
    //             let z = self.zoom * (1.1 as f32).powf(-v);
    //             let pos = self.mouse_state.last_pos;
    //             self.zoom_to_center(pos,if z >= 1.0 { z } else { 1.0 } );
    //         },
    //         _ => {}
    //     }
    // }

    fn update_annotations(&mut self) {
        let maxw : usize = 16384;
        let tw : usize = maxw;
        let th : usize = (self.data_len + (maxw-1))/maxw;
        let cloned_annot = self.annotation_d.clone();
        // let annotation_img = glium::texture::RawImage2d {
        //     data : std::borrow::Cow::Borrowed(cloned_annot.as_slice()),
        //     width : tw as u32,
        //     height : th as u32,
        //     format : glium::texture::ClientFormat::U8,
        // };
        // let annotation_tex = glium::texture::UnsignedTexture2d::with_mipmaps(&self.display, annotation_img, glium::texture::MipmapsOption::NoMipmap).unwrap();
        // self.annotation_tex = annotation_tex;
    }
/*
    // Handle keyboard input
    fn handle_kb(&mut self, input : KeyboardInput) {
        match input {
            KeyboardInput { scancode:_, state:glutin::ElementState::Pressed,
                            virtual_keycode:Some(vkeycode),modifiers:_ } =>
                match vkeycode {
                    glutin::VirtualKeyCode::Escape => self.closed = true,
                    glutin::VirtualKeyCode::Right => self.zoom_in(),
                    glutin::VirtualKeyCode::Left => self.zoom_out(),
                    glutin::VirtualKeyCode::S => {
                        use annotation::AnnotationEngine;
                        let engine = annotation::CStringAnnotationEngine::new();
                        let annotations = engine.build_annotations(self.dat);
                        for annotation in annotations.iter() {
                            for n in annotation.span().0 .. annotation.span().1 {
                                self.annotation_d[n] = 0x66;
                            }
                        }
                        self.annotation_store = Some(annotations);
                        self.update_annotations();
                    },
                    _ => (),
                },
            _ => (),
        }
    }

    fn handle_mouse_move(&mut self, pos : (f64, f64) ) {
        use self::glutin::MouseButton::*;
        self.mouse_state.last_pos = pos;
        if !self.mouse_state.down.is_empty() {
            if self.mouse_state.down.contains(&Left) {
                let drag_end = self.byte_from_coords(self.mouse_state.last_pos);
                let drag_start = match self.mouse_state.start_drag_pos {
                    None => None, Some(x) => self.byte_from_coords(x),
                };
                match (drag_start, drag_end) {
                    (Some(s), Some(e)) => self.set_selection(s * 8, e * 8), // *8 because bit index
                    _ => {},
                }
            } else if self.mouse_state.down.contains(&Middle) {
                let (x1, y1) = self.mouse_state.start_drag_pos.unwrap();
                let (x2, y2) = self.mouse_state.last_pos;
                let (dx, dy) = (x2 - x1, y2 - y1);
                let xoff = self.mouse_state.start_ul_offset.0 - dx as f32;
                let yoff = self.mouse_state.start_ul_offset.1 - dy as f32;
                self.ul_offset = (xoff, yoff);
            }
        }
    }
  */  
    
    fn byte_from_coords(&self, pos : (f64, f64) ) -> Option<u32> {
        // find (possibly off-screen) location of 0,0 in data.
        // adjust for zoom
        let (x, y) = ((pos.0 + self.ul_offset.0 as f64)/self.zoom as f64,
                      (pos.1 + self.ul_offset.1 as f64)/self.zoom as f64);
        // add deltas to upper left corner of image
        
        if x < 0.0 || y < 0.0 || y >= self.col_height as f64
        {
            None
        } else {
            let column = x as u32/self.stride;
            let row = y as u32;
            let idx = column * self.col_height + row;
            if idx < self.data_len as u32 { Some(idx) } else { None }
        }
    }
/*
    fn handle_mouse_button(&mut self, state : glutin::ElementState, button : glutin::MouseButton ) {
        use self::glutin::MouseButton::*;
        use self::glutin::ElementState::*;

        match button {
            Left => {
                // No commit on mouse release; selection is updated during drag
            },
            Middle => match state {
                Pressed => {
                    self.mouse_state.start_ul_offset = self.ul_offset;
                },
                _ => {},
            },
            _ => {},
        }
        
        match state {
            Pressed => {
                self.mouse_state.down.insert(button);
                self.mouse_state.start_drag_pos = Some(self.mouse_state.last_pos);
            },
            Released => {
                self.mouse_state.down.remove(&button);
                self.mouse_state.start_drag_pos = None;
            },
        };

    }
*/
        
    pub fn handle_events(&mut self) {
        /*
        let mut evec : Vec<glium::glutin::Event> = Vec::new();
        self.events.poll_events(|event| { evec.push(event); });
        for event in evec {
            match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::Closed => self.closed = true,
                    glutin::WindowEvent::KeyboardInput {input , ..} => self.handle_kb(input),
                    glutin::WindowEvent::MouseMoved {position, .. } => self.handle_mouse_move(position),
                    glutin::WindowEvent::MouseWheel {delta, .. } =>self.handle_mouse_scroll(delta),
                    glutin::WindowEvent::MouseInput {state, button, .. } => self.handle_mouse_button(state,button),
                    _ => ()
                },
                _ => (),
            }
        };
*/
    }
}
