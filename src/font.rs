extern crate glium;

use std;

use glium::Surface;

// Shader sources
static VS_SRC: &'static str = include_str!("fontvs.glsl");
static FS_SRC: &'static str = include_str!("fontfs.glsl");

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

pub struct Font {
    font_tex : glium::texture::UnsignedTexture2d,
    program : glium::Program,
    params : glium::DrawParameters<'static>,
    zoom_factor : f32,
}

impl Font {
    pub fn new(display : &glium::Display) -> Font {
        use glium::texture::*;
        let font_img = RawImage2d {
            data : std::borrow::Cow::Borrowed(include_bytes!("fonts/Osborne_I.charrom.2")),
            width : 1024,
            height : 10,
            format : ClientFormat::U8,
        };
        let font_tex = UnsignedTexture2d::with_mipmaps(display,
                                                       font_img,
                                                       MipmapsOption::NoMipmap).unwrap();
        let program = glium::Program::from_source(display,
                                                  VS_SRC,
                                                  FS_SRC,
                                                  None)
            .expect("Failed to create shader program");
        let params = {
            use glium::BlendingFunction::Addition;
            use glium::LinearBlendingFactor::*;
            
            let blending_function = Addition {
                source: SourceAlpha,
                destination: OneMinusSourceAlpha
            };
            
            let blend = glium::Blend {
                color: blending_function,
                alpha: blending_function,
                constant_value: (1.0, 1.0, 1.0, 1.0),
            };
            
            glium::DrawParameters {
                blend: blend,
                .. Default::default()
            }
        };
        Font {
            font_tex : font_tex,
            program : program,
            params : params,
            zoom_factor : 2.0,
        }
    }

    pub fn width(&self,text:&str) -> u32 { (self.zoom_factor*8.0*text.len() as f32) as u32 }
    pub fn height(&self) -> u32 { (self.zoom_factor*10.0) as u32 }

    pub fn draw(&self, 
                display : &glium::Display, 
                frame : &mut glium::Frame, 
                window_size : (u32, u32),
                text_pos : (u32, u32),
                text : &str) {
        let charw = 1.0 / 128.0;
        // chars are 8x8
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
        let indices = glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList,
                                              &INDICES).unwrap();
        for b in text.as_bytes() {
            let tex_left = charw * *b as f32;

            // Two triangles to cover the character
            let vertices : [Vertex; 4] = [
                Vertex{ position : [x,y+ch], tex_coords : [tex_left,0.0] },
                Vertex{ position : [x+cw,y+ch], tex_coords : [tex_left+charw,0.0] },
                Vertex{ position : [x+cw,y], tex_coords : [tex_left+charw,1.0] },
                Vertex{ position : [x,y], tex_coords : [tex_left,1.0] },
            ];
            let positions = glium::VertexBuffer::new(display, &vertices).unwrap();
            
            let uniforms = uniform! {
                in_color : (0.0 as f32, 1.0 as f32, 0.0 as f32, 0.9 as f32),
                tex : &self.font_tex,
            };
            
            frame.draw(&positions, &indices, &self.program,
                       &uniforms, &self.params).unwrap();
            x = x + cw;
        }
        
    }
}

