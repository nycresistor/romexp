extern crate glium;

use glium::{glutin, Surface};

use std;
use std::mem;
use std::ptr;
use std::str;
use std::ffi::CString;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

// Two triangles to cover the window
const VERTICES : [Vertex; 4] = [
    Vertex{ position : [-1.0,1.0] },
    Vertex{ position : [1.0,1.0] },
    Vertex{ position : [1.0,-1.0] },
    Vertex{ position : [-1.0,-1.0] },
];

const INDICES : [u16; 6] = [
    0, 1, 2,
    0, 3, 2,
];

// Shader sources
static VS_SRC: &'static str = include_str!("vs.glsl");

static FS_SRC: &'static str = include_str!("fs.glsl");

pub struct Visualizer {
    pub events : glutin::EventsLoop,
    display : glium::Display,
    program : glium::Program,
    positions : glium::VertexBuffer<Vertex>,
    indices : glium::IndexBuffer<u16>,
    data_len : usize,
    stride : u32,
    size : (u32, u32),
    selection : (u32, u32),
    texture : glium::texture::UnsignedTexture2d,
    pub closed : bool,
}

impl Visualizer {
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
        let teximg = glium::texture::RawImage2d {
            data : std::borrow::Cow::Borrowed(d.as_slice()),
            width : tw as u32,
            height : th as u32,
            format : glium::texture::ClientFormat::U8,
        };
        let texture = glium::texture::UnsignedTexture2d::with_mipmaps(&display, teximg, glium::texture::MipmapsOption::NoMipmap).unwrap();

        let mut vz = Visualizer {
            events : events_loop,
            display : display,
            program : program,
            positions : positions,
            indices : indices,
            data_len : dat.len(),
            stride : 8,
            selection : (0,0),
            texture : texture,
            size : size,
            closed: false,
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
            colstride : self.stride*512,
            datalen : self.data_len as u32,
            selection : self.selection,
            texwidth : 16384 as u32,
            romtex : &self.texture,
        };
            
        target.draw(&self.positions, &self.indices, &self.program,
                    &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();
    }

    pub fn handle_events(&mut self) {
        let mut evec : Vec<glium::glutin::Event> = Vec::new();
        self.events.poll_events(|event| { evec.push(event); });
        for event in evec {
            match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::Closed => self.closed = true,
                    _ => ()
                },
                _ => (),
            }
        };
    }
}
