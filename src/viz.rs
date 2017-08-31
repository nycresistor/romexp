extern crate glium;

use glium::{glutin, Surface};

use std;
use std::str;

use glium::glutin::KeyboardInput;

use annotation;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

// Two triangles to cover the window
const VERTICES : [Vertex; 4] = [
    Vertex{ position : [-1.0,1.0], tex_coords : [0.0,1.0] },
    Vertex{ position : [1.0,1.0], tex_coords : [1.0,1.0] },
    Vertex{ position : [1.0,-1.0], tex_coords : [1.0,0.0] },
    Vertex{ position : [-1.0,-1.0], tex_coords : [0.0,0.0] },
];

const INDICES : [u16; 6] = [
    0, 1, 2,
    0, 3, 2,
];

// Shader sources
static VS_SRC: &'static str = include_str!("vs.glsl");

static FS_SRC: &'static str = include_str!("fs.glsl");

use std::collections::HashSet;

pub struct MouseState {
    start_drag_pos : Option<(f64,f64)>,
    last_pos : (f64, f64),
    down : HashSet<glutin::MouseButton>,
    start_ul_offset : (f32, f32),
}

impl MouseState {
    pub fn new() -> MouseState {
        MouseState { 
            start_drag_pos : None, 
            last_pos : (0.0, 0.0),
            down : HashSet::new(),
            start_ul_offset : (0.0, 0.0),
        }
    }
}

pub struct Visualizer<'a> {
    events : glutin::EventsLoop,
    display : glium::Display,
    program : glium::Program,
    positions : glium::VertexBuffer<Vertex>,
    indices : glium::IndexBuffer<u16>,
    data_len : usize,
    /// width, in bits, of each column
    stride : u32,
    /// height, in rows, of each colum
    col_height : u32,
    /// width and height of window, in px
    size : (u32, u32),
    /// start and end of current selection, as byte idx
    selection : (u32, u32),
    texture : glium::texture::UnsignedTexture2d,
    annotation_tex : glium::texture::UnsignedTexture2d,
    annotation_d : Vec<u8>,
    zoom : f32,
    ul_offset : (f32, f32),
    pub closed : bool,
    mouse_state : MouseState,
    dat : &'a [u8],
}

impl<'a> Visualizer<'a> {
    pub fn new(size : (u32, u32), dat : &[u8]) -> Visualizer {
        let mut events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_title("ROM Explorer")
            .with_dimensions(size.0, size.1);
        let context = glutin::ContextBuilder::new();
        let display = glium::Display::new(window, context, &events_loop)
            .expect("Failed to create Glium window.");

        let program = glium::Program::from_source(&display, VS_SRC, FS_SRC, None)
            .expect("Failed to create shader program");

        let positions = glium::VertexBuffer::new(&display, &VERTICES).unwrap();
        let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList,
                                              &INDICES).unwrap();
        

        let maxw : usize = 16384;
        let tw : usize = maxw;
        let th : usize = (dat.len() + (maxw-1))/maxw;
        let mut d : Vec<u8> = Vec::new();
        d.reserve(tw*th);
        d.extend(dat.iter().cloned());
        d.resize(tw*th,0);
        let teximg = glium::texture::RawImage2d {
            data : std::borrow::Cow::Borrowed(d.as_slice()),
            width : tw as u32,
            height : th as u32,
            format : glium::texture::ClientFormat::U8,
        };
        let mut annotation_d : Vec<u8> = Vec::new();
        annotation_d.resize(tw*th,0);
        let cloned_annot = annotation_d.clone();
        let annotation_img = glium::texture::RawImage2d {
            data : std::borrow::Cow::Borrowed(cloned_annot.as_slice()),
            width : tw as u32,
            height : th as u32,
            format : glium::texture::ClientFormat::U8,
        };
        let texture = glium::texture::UnsignedTexture2d::with_mipmaps(&display, teximg, glium::texture::MipmapsOption::NoMipmap).unwrap();
        let annotation_tex = glium::texture::UnsignedTexture2d::with_mipmaps(&display, annotation_img, glium::texture::MipmapsOption::NoMipmap).unwrap();

        let mut vz = Visualizer {
            events : events_loop,
            display : display,
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
            size : size,
            zoom : 1.0,
            ul_offset : (0.0, 0.0),
            closed: false,
            mouse_state : MouseState::new(),
            dat : dat,
        };
        vz.set_size(size);
        vz
    }
    
    pub fn set_selection(&mut self, start : u32, finish : u32) {
        self.selection = (start, finish);
    }

    pub fn set_size(&mut self, size : (u32, u32)) {
        self.size = size;
    }

    pub fn set_stride(&mut self, stride : u32) {
        self.stride = stride;
    }

    pub fn render(&mut self) {
        let mut target = self.display.draw();
        target.clear_color(1.0,0.0,0.0,1.0);
        let uniforms = uniform! {
            win : [ 0, 0, self.size.0, self.size.1 ],
            bitstride : self.stride,
            colstride : self.stride*self.col_height,
            datalen : self.data_len as u32,
            selection : self.selection,
            texwidth : 16384 as u32,
            romtex : &self.texture,
            annotex : &self.annotation_tex,
            ul_offset : self.ul_offset,
            zoom : self.zoom,
        };
            
        target.draw(&self.positions, &self.indices, &self.program,
                    &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();
    }

    fn zoom_to(&mut self, z : f32) {
        fn findul(ul : f32, win : u32, oldz : f32, newz : f32) -> f32 {
            let half = win as f32 / 2.0;
            let c = ul + (half / oldz);
            c - half / newz
        }
        self.ul_offset = ( findul(self.ul_offset.0, self.size.0, self.zoom, z),
                          findul(self.ul_offset.1, self.size.1, self.zoom, z) );
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

    fn handle_mouse_scroll(&mut self, d : glutin::MouseScrollDelta) {
        match d {
            glutin::MouseScrollDelta::LineDelta(h,v) => {
                let z = self.zoom - 0.1 * v;
                self.zoom_to(if z >= 1.0 { z } else { 1.0 } );
            },
            _ => {}
        }
    }

    fn update_annotations(&mut self) {
        let maxw : usize = 16384;
        let tw : usize = maxw;
        let th : usize = (self.data_len + (maxw-1))/maxw;
        let cloned_annot = self.annotation_d.clone();
        let annotation_img = glium::texture::RawImage2d {
            data : std::borrow::Cow::Borrowed(cloned_annot.as_slice()),
            width : tw as u32,
            height : th as u32,
            format : glium::texture::ClientFormat::U8,
        };
        let annotation_tex = glium::texture::UnsignedTexture2d::with_mipmaps(&self.display, annotation_img, glium::texture::MipmapsOption::NoMipmap).unwrap();
        self.annotation_tex = annotation_tex;
    }

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
                        for annotation in annotations {
                            for n in annotation.span().0 .. annotation.span().1 {
                                self.annotation_d[n] = 0x66;
                            }
                        }
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
        if self.mouse_state.down.contains(&Left) {
            let drag_end = self.byte_from_coords(self.mouse_state.last_pos);
            let start_idx = self.byte_from_coords(self.mouse_state.start_drag_pos.unwrap()).unwrap();
            let end_idx = drag_end.unwrap();
            self.set_selection(start_idx * 8, end_idx * 8); // *8 because bit index
        } else if self.mouse_state.down.contains(&Middle) {
            let (x1, y1) = self.mouse_state.start_drag_pos.unwrap();
            let (x2, y2) = self.mouse_state.last_pos;
            let (dx, dy) = (x2 - x1, y2 - y1);
            let xoff = self.mouse_state.start_ul_offset.0 - dx as f32;
            let yoff = self.mouse_state.start_ul_offset.1 - dy as f32;
            self.ul_offset = (xoff, yoff);
        }
    }
    
    
    fn byte_from_coords(&self, pos : (f64, f64) ) -> Option<u32> {
        // find (possibly off-screen) location of 0,0 in data.
        // adjust for zoom
        let (x, y) = ((pos.0 + self.ul_offset.0 as f64)/self.zoom as f64,
                      (pos.1 + self.ul_offset.1 as f64)/self.zoom as f64);
        // add deltas to upper left corner of image
        
        if x < 0.0 || y < 0.0
        {
            Some(0)
        } else {
            let column = x as u32/self.stride;
            let row = y as u32;
            let idx = column * self.col_height + row;
            if idx < self.data_len as u32 { Some(idx) } else { Some(self.data_len as u32) }
        }
    }

    fn handle_mouse_button(&mut self, state : glutin::ElementState, button : glutin::MouseButton ) {
        use self::glutin::MouseButton::*;
        use self::glutin::ElementState::*;

        match button {
            Left => match state {
                Released => {
                    let drag_end = self.byte_from_coords(self.mouse_state.last_pos);
                    let start_idx = self.byte_from_coords(self.mouse_state.start_drag_pos.unwrap()).unwrap();
                    let end_idx = drag_end.unwrap();
                    self.set_selection(start_idx*8, end_idx*8); // *8 because bit index
                },
                _ => {},
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

        
    pub fn handle_events(&mut self) {
        let mut evec : Vec<glium::glutin::Event> = Vec::new();
        self.events.poll_events(|event| { evec.push(event); });
        for event in evec {
            match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::Closed => self.closed = true,
                    glutin::WindowEvent::Resized( w, h ) => self.set_size((w,h)),
                    glutin::WindowEvent::KeyboardInput {input , ..} => self.handle_kb(input),
                    glutin::WindowEvent::MouseMoved {position, .. } => self.handle_mouse_move(position),
                    glutin::WindowEvent::MouseWheel {delta, .. } =>self.handle_mouse_scroll(delta),
                    glutin::WindowEvent::MouseInput {state, button, .. } => self.handle_mouse_button(state,button),
                    _ => ()
                },
                _ => (),
            }
        };
    }
}
