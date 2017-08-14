extern crate glium;

use glium::{glutin, Surface};
use imgui;
use imgui::{ImGui, Ui};
use imgui_glium_renderer::Renderer;
use std;
use std::str;

use glium::glutin::KeyboardInput;
use std::time::Instant;

#[derive(Copy, Clone, PartialEq, Debug, Default)]
struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
}

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

pub struct Visualizer {
    display : glium::Display,
    program : glium::Program,
    positions : glium::VertexBuffer<Vertex>,
    indices : glium::IndexBuffer<u16>,
    data_len : usize,
    stride : u32,
    size : (u32, u32),
    selection : (u32, u32),
    texture : glium::texture::UnsignedTexture2d,
    zoom : f32,
    pub closed : bool,
    imgui : ImGui,
    renderer : Renderer,
    mouse_state : MouseState,
}

impl Visualizer {
    pub fn new(size : (u32, u32), events_loop : &mut glutin::EventsLoop, dat : &[u8]) -> Visualizer {
        // Create glium display and program for bit visualizer.
        let window = glutin::WindowBuilder::new()
            .with_title("ROM Explorer")
            .with_dimensions(size.0, size.1);
        let context = glutin::ContextBuilder::new();
        let display = glium::Display::new(window, context, events_loop)
            .expect("Failed to create Glium window.");

        let program = glium::Program::from_source(&display, VS_SRC, FS_SRC, None)
            .expect("Failed to create shader program");
        // Set up simple square from -.5,-.5 to .5,.5 for bit display.
        let positions = glium::VertexBuffer::new(&display, &VERTICES).unwrap();
        let indices = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList,
                                              &INDICES).unwrap();
        // Convert binary blob to texture for shader consumption.
        // To allow blobs larger than 16K, we use a 2D texture instead of
        // the more obvious 1D. (This gives us a theoretical quarter-gig.)
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

        // Set up and initialize an ImGui renderer for informational display.
        let mut imgui = ImGui::init();
        let renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize ImGui renderer");

        let mut vz = Visualizer {
            display : display,
            program : program,
            positions : positions,
            indices : indices,
            data_len : dat.len(),
            stride : 8,
            selection : (0,0),
            texture : texture,
            size : size,
            zoom : 1.0,
            closed : false,
            imgui : imgui,
            renderer : renderer,
            mouse_state : MouseState::default(),
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


    fn zoom_in(&mut self) {
        self.zoom = if self.zoom >= 1.0 { self.zoom + 1.0 } else { 1.0 };
    }

    fn zoom_out(&mut self) {
        self.zoom = if self.zoom > 1.0 { self.zoom - 1.0 } else { self.zoom / 2.0 };
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
                    _ => (),
                },
            _ => (),
        }
    }
}

impl super::Vizwin for Visualizer {
    fn render(&mut self) {
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
            zoom : self.zoom,
        };
            
        target.draw(&self.positions, &self.indices, &self.program,
                    &uniforms, &Default::default()).unwrap();

        let gl_window = self.display.gl_window();
        let size_points = gl_window.get_inner_size_points().unwrap();
        let size_pixels = gl_window.get_inner_size_pixels().unwrap();

        let ui = self.imgui.frame(size_points, size_pixels, 0.01);
        ui.window(im_str!("Hello world"))
            .size((300.0, 100.0), imgui::ImGuiSetCond_FirstUseEver)
            .build(|| {
                ui.text(im_str!("Hello world!"));
                ui.text(im_str!("This...is...imgui-rs!"));
                ui.separator();
                let mouse_pos = ui.imgui().mouse_pos();
                ui.text(im_str!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos.0,
                    mouse_pos.1
                ));
            });
        self.renderer.render(&mut target, ui).expect("ImGui rendering failed");
        
        target.finish().unwrap();
    }
    
    fn handle_events(&mut self, events : &mut glutin::EventsLoop) {
        let mut evec : Vec<glium::glutin::Event> = Vec::new();
        events.poll_events(|event| { evec.push(event); });
        use glium::glutin::WindowEvent::*;
        use glium::glutin::ElementState::Pressed;
        use glium::glutin::{Event, MouseButton, MouseScrollDelta, TouchPhase};
        for event in evec {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    Closed => self.closed = true,
                    KeyboardInput {input , ..} => self.handle_kb(input),
                    MouseMoved { position: (x, y), .. } => self.mouse_state.pos = (x as i32, y as i32),
                    MouseInput { state, button, .. } => {
                        match button {
                            MouseButton::Left => self.mouse_state.pressed.0 = state == Pressed,
                            MouseButton::Right => self.mouse_state.pressed.1 = state == Pressed,
                            MouseButton::Middle => self.mouse_state.pressed.2 = state == Pressed,
                            _ => {}
                        }
                    },
                    MouseWheel {
                        delta: MouseScrollDelta::LineDelta(_, y),
                        phase: TouchPhase::Moved,
                        ..
                    } |
                    MouseWheel {
                        delta: MouseScrollDelta::PixelDelta(_, y),
                        phase: TouchPhase::Moved,
                        ..
                    } => self.mouse_state.wheel = y,                
                    _ => ()
                },
                _ => (),
            }
        };
        let scale = self.imgui.display_framebuffer_scale();
        self.imgui.set_mouse_pos(
            self.mouse_state.pos.0 as f32 / scale.0,
            self.mouse_state.pos.1 as f32 / scale.1,
        );
        self.imgui.set_mouse_down(
            &[
                self.mouse_state.pressed.0,
                self.mouse_state.pressed.1,
                self.mouse_state.pressed.2,
                false,
                false,
            ],
        );
        self.imgui.set_mouse_wheel(self.mouse_state.wheel / scale.1);
        self.mouse_state.wheel = 0.0;    
    }
}
