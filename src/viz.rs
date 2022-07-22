extern crate glfw;

use gl;
use gl::types::*;
use glfw::{Context, Window, WindowEvent};

use std;

use annotation;
use font;
use glutil;

// A DataView describes what portion of the data texture is rendered on-screen, and how it is
// laid out.
pub struct DataView {
    data_bounds: (usize, usize), // Start and endpoints of rendered data
    column_dim: (u32, u32),      // The width and height of individual columns of data, in pixels
    column_spacing: u32,         // The space, in pixels, between adjacent columns
    bits_per_pixel: u8,          // The number of bits to render per picture element
                                 // endianness is not yet implemented, but would be part of a dataview
}

impl DataView {
    /// Compute the width of a column, in bytes.
    pub fn byte_width(&self) -> u32 {
        return self.column_dim.0 / (8 * self.bits_per_pixel) as u32;
    }
    /// Compute length of data displayed.
    pub fn data_len(&self) -> usize {
        return self.data_bounds.1 - self.data_bounds.0;
    }
}

// The ViewState describes how the data is viewed visually: panning, zooming, etc.
pub struct ViewState {
    zoom: f32,             // The zoom level
    ul_offset: (f32, f32), // The offset of the top left corner of the view relative to the
    // top left corner of the window, in pre-zoomed pixels
    selection: (u32, u32), // The current area selected, as indexes into the data texture
}

// Shader sources
static VS_SRC: &'static str = include_str!("vs.glsl");

static FS_SRC: &'static str = include_str!("fs.glsl");

// Dragging is a stateful mouse interaction.
enum MouseDragOp {
    NoOp,
    Select {
        start: (f64, f64),
    },
    Panning {
        original_ul: (f32, f32),
        start: (f64, f64),
    },
}

pub struct MouseState {
    last_pos: (f64, f64),
    moved: bool, //< Whether we've actually dragged or just clicked
    op: MouseDragOp,
}

impl MouseState {
    pub fn new() -> MouseState {
        MouseState {
            last_pos: (0.0, 0.0),
            moved: false,
            op: MouseDragOp::NoOp,
        }
    }
}

/// The top-level visualizer window.
pub struct Visualizer<'a> {
    pub window: Window,
    pub events: std::sync::mpsc::Receiver<(f64, WindowEvent)>,

    // GL state
    program: GLuint,        // the GL program, consisting of shaders.
    vao: GLuint,            // the vertex array object.
    texture: GLuint,        // the data being examined.
    annotation_tex: GLuint, // the annotation data to display.

    dview: DataView,
    vstate: ViewState,
    annotation_d: Vec<u8>,
    pub closed: bool,       // Whether the window has been closed; true at shutdown.
    mouse_state: MouseState,
    dat: &'a [u8],
    annotation_store: Option<annotation::AnnotationStore>,
    font: font::Font,
}

// The vertices for the two triangles that make up the window. Each vertex is composed of
// four values. The first pair is the position of the corners of the triangles in 3d space
// (the Z coordinate is always 0.0). and the second pair is the location of that same point
// in the texture coordinate space. (Thus, the upper left corner is (0,0), which is the first
// byte of the displayed data.
const VERTICES: [GLfloat; 16] = [
    -1.0, 1.0,   0.0, 1.0,
     1.0, 1.0,   1.0, 1.0,
     1.0, -1.0,  1.0, 0.0,
    -1.0, -1.0,  0.0, 0.0,
];

// The indices of the vertexes of two triangles which cover the window.
const INDICES: [GLuint; 6] = [
    0, 1, 2,
    0, 3, 2
];

impl<'a> Visualizer<'a> {
    pub fn new(glfw: &mut glfw::Glfw, size: (u32, u32), dat: &'a [u8]) -> Visualizer<'a> {
        let (mut window, events) = glfw
            .create_window(size.0, size.1, "ROM Explorer", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");
        gl::load_with(|s| window.get_proc_address(s) as *const _);
        let program = glutil::build_program(VS_SRC, FS_SRC).unwrap();
        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);
        window.set_scroll_polling(true);
        window.set_size_polling(true);
        let mut vbo: GLuint = 0; // The ID of the vertex buffer object, referencing the coordinates.
        let mut ebo: GLuint = 0; // The ID of the element buffer object, referencing the indices.
        let mut vao: GLuint = 0; // The ID of the vertex array object, which collects the above.
        unsafe {
            // Bind our shader programs
            gl::UseProgram(program);
            /* This is not necessary unless you have more than one "out" variable.
            gl::BindFragDataLocation(
                program,
                0,
                std::ffi::CString::new("color").unwrap().as_ptr(),
            );
            */

            // Create the vertex array object, and add all the data necessary to
            // draw the triangles.
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                4 * 16,
                VERTICES.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                4 * 6,
                INDICES.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let pos_attrib = glutil::attrib_loc(program, "position") as GLuint;
            gl::EnableVertexAttribArray(pos_attrib);
            gl::VertexAttribPointer(pos_attrib, 2, gl::FLOAT, gl::FALSE, 4 * 4, std::ptr::null());
            let tex_attrib = glutil::attrib_loc(program, "tex_coords") as GLuint;
            gl::EnableVertexAttribArray(tex_attrib as GLuint);
            gl::VertexAttribPointer(
                tex_attrib,
                2,
                gl::FLOAT,
                gl::FALSE,
                4 * 4,
                (2 * 4) as *const _,
            );
        }
        let maxw: usize = 16384;
        let tw: usize = maxw;
        let th: usize = (dat.len() + (maxw - 1)) / maxw;
        let mut d: Vec<u8> = Vec::new();
        d.reserve(tw * th);
        d.extend(dat.iter().cloned());
        d.resize(tw * th, 0);

        let mut texture: GLuint = 0;
        let mut annotation_tex: GLuint = 0;
        let mut annotation_d: Vec<u8> = Vec::new();
        annotation_d.resize(tw * th, 0);
        let cloned_annot = annotation_d.clone();
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::R8UI as GLint,
                tw as GLsizei,
                th as GLsizei,
                0,
                gl::RED_INTEGER,
                gl::UNSIGNED_BYTE,
                d.as_ptr() as *const GLvoid,
            );
            gl::GenTextures(1, &mut annotation_tex);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, annotation_tex);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::R8UI as GLint,
                tw as GLsizei,
                th as GLsizei,
                0,
                gl::RED_INTEGER,
                gl::UNSIGNED_BYTE,
                cloned_annot.as_ptr() as *const GLvoid,
            );

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, annotation_tex);
        }

        Visualizer {
            window: window,
            events: events,
            program: program,
            vao: vao,
            dview: DataView {
                data_bounds: (0, dat.len()),
                column_dim: (8, 512),
                column_spacing: 4,
                bits_per_pixel: 1,
            },
            vstate: ViewState {
                zoom: 1.0,
                ul_offset: (0.0, 0.0),
                selection: (0, 0),
            },
            texture: texture,
            annotation_tex: annotation_tex,
            annotation_d: annotation_d,
            closed: false,
            mouse_state: MouseState::new(),
            dat: dat,
            annotation_store: None,
            font: font::Font::new(),
        }
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.dview.data_bounds.0 = offset;
    }

    pub fn set_selection(&mut self, start: u32, finish: u32) {
        self.vstate.selection = (start, finish);
    }

    pub fn set_col_width(&mut self, width: u32) {
        self.dview.column_dim.0 = width;
    }

    pub fn set_spacing(&mut self, spacing: u32) {
        self.dview.column_spacing = spacing;
    }

    pub fn uniloc(&self, name: &str) -> GLint {
        let c_str = std::ffi::CString::new(name.as_bytes()).unwrap();
        let loc = unsafe { gl::GetUniformLocation(self.program, c_str.as_ptr()) };
        loc
    }

    pub fn render(&mut self) {
        let size = self.window.get_size();
        unsafe {
            gl::UseProgram(self.program);
            gl::ClearColor(0.5, 0.0, 0.0, 1.0);
            //gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.annotation_tex);

            gl::Uniform4ui(self.uniloc("win"), 0, 0, size.0 as u32, size.1 as u32);
            gl::Uniform1ui(self.uniloc("colwidth"), self.dview.column_dim.0);
            gl::Uniform1ui(self.uniloc("colheight"), self.dview.column_dim.1);
            gl::Uniform1ui(self.uniloc("colspace"), self.dview.column_spacing);
            gl::Uniform1ui(
                self.uniloc("datalen"),
                (self.dview.data_bounds.1 - self.dview.data_bounds.0) as u32,
            );
            gl::Uniform1ui(self.uniloc("dataoff"), self.dview.data_bounds.0 as u32);
            gl::Uniform1ui(self.uniloc("bpp"), self.dview.bits_per_pixel as u32);
            gl::Uniform2ui(self.uniloc("selection"), self.vstate.selection.0, self.vstate.selection.1);
            gl::Uniform1ui(self.uniloc("texwidth"), 16384 as u32);
            gl::Uniform1i(self.uniloc("romtex"), 0 as i32); //self.texture as i32);
            gl::Uniform1i(self.uniloc("annotex"), 1 as i32); //self.annotation_tex as i32);
            gl::Uniform2f(self.uniloc("ul_offset"), self.vstate.ul_offset.0, self.vstate.ul_offset.1);
            gl::Uniform1f(self.uniloc("zoom"), self.vstate.zoom);

            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
        }
        let bfc = self.byte_from_coords(self.mouse_state.last_pos);
        {
            let text = match bfc {
                Some(x) => format!("0x{:x} ({:x})", x, x % (self.dview.byte_width())),
                None => String::new(),
            };
            let text_sz = self.font.size(text.as_str());
            let location = (size.0 - text_sz.0 as i32, size.1 - text_sz.1 as i32);
            self.font.draw(size, location, text.as_str());
        }
        {
            let status = format!("str 0x{:x}", self.dview.byte_width());
            let text_sz = self.font.size(status.as_str());
            let location = (size.0 - text_sz.0 as i32, size.1 - 2 * text_sz.1 as i32);
            self.font.draw(size, location, status.as_str());
        }
        match bfc {
            Some(x) => match self.annotation_store {
                Some(ref store) => {
                    let annos = store.query(x as usize);
                    let y = 0;
                    for a in annos {
                        let s = a.comments();
                        let location = (size.0.saturating_sub(self.font.width(s)), y);
                        self.font.draw(size, location, s);
                    }
                }
                None => {}
            },
            None => {}
        }
        self.window.swap_buffers();
    }

    fn zoom_to_center(&mut self, cursor: (f64, f64), z: f32) {
        fn findul(ul: f32, cursor: f32, oldz: f32, newz: f32) -> f32 {
            // convert all coords to zoom level 1.0
            let ul1 = ul / oldz;
            let cursor1 = cursor / oldz;
            let ulnew1 = ul1 + (cursor1 * (1.0 - 1.0 / (newz / oldz)));
            ulnew1 * newz
        }
        self.vstate.ul_offset = (
            findul(self.vstate.ul_offset.0, cursor.0 as f32, self.vstate.zoom, z),
            findul(self.vstate.ul_offset.1, cursor.1 as f32, self.vstate.zoom, z),
        );
        self.vstate.zoom = z;
    }

    fn zoom_to(&mut self, z: f32) {
        fn findul(ul: f32, win: u32, oldz: f32, newz: f32) -> f32 {
            let half = win as f32 / 2.0;
            let c = ul + (half / oldz);
            c - half / newz
        }
        let size = self.window.get_size();
        self.vstate.ul_offset = (
            findul(self.vstate.ul_offset.0, size.0 as u32, self.vstate.zoom, z),
            findul(self.vstate.ul_offset.1, size.1 as u32, self.vstate.zoom, z),
        );
        self.vstate.zoom = z;
    }

    fn zoom_in(&mut self) {
        let z = if self.vstate.zoom >= 1.0 {
            self.vstate.zoom + 0.1
        } else {
            1.0
        };
        self.zoom_to(z);
    }

    fn zoom_out(&mut self) {
        let z = if self.vstate.zoom > 1.0 {
            self.vstate.zoom - 0.1
        } else {
            1.0
        };
        self.zoom_to(z);
    }

    fn handle_scroll(&mut self, ydelta: f64) {
        let z = self.vstate.zoom * (1.1 as f32).powf(ydelta as f32);
        let pos = self.mouse_state.last_pos;
        self.zoom_to_center(pos, if z >= 1.0 { z } else { 1.0 });
    }

    fn update_annotations(&mut self) {
        let maxw: usize = 16384;
        let tw: usize = maxw;
        let th: usize = (self.dview.data_len() + (maxw - 1)) / maxw;
        let cloned_annot = self.annotation_d.clone();
        unsafe {
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.annotation_tex);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::R8UI as GLint,
                tw as GLsizei,
                th as GLsizei,
                0,
                gl::RED_INTEGER,
                gl::UNSIGNED_BYTE,
                cloned_annot.as_ptr() as *const GLvoid,
            );
        }
    }

    // Handle keyboard input
    fn handle_kb(&mut self, key: glfw::Key) {
        use glfw::Key::*;
        match key {
            Num1 => self.dview.bits_per_pixel = 1,
            Num2 => self.dview.bits_per_pixel = 2,
            Num4 => self.dview.bits_per_pixel = 4,
            Num8 => self.dview.bits_per_pixel = 8,

            Escape => self.window.set_should_close(true),
            Up => self.zoom_in(),
            Down => self.zoom_out(),
            Right => {
                let s = self.dview.column_dim.0 + (8 / self.dview.bits_per_pixel) as u32;
                self.set_col_width(s);
            }
            Left => {
                let s = self.dview.column_dim.0 - (8 / self.dview.bits_per_pixel) as u32;
                self.set_col_width(if s < 8 { 8 } else { s });
            }
            GraveAccent => {
                // nop until we do endiannesss
            }
            S => {
                use annotation::AnnotationEngine;
                let engine = annotation::CStringAnnotationEngine::new();
                let annotations = engine.build_annotations(self.dat);
                for annotation in annotations.iter() {
                    for n in annotation.span().0..annotation.span().1 {
                        self.annotation_d[n] = 0x66;
                    }
                }
                self.annotation_store = Some(annotations);
                self.update_annotations();
            }
            _ => (),
        }
    }

    fn handle_mouse_move(&mut self, pos: (f64, f64)) {
        if self.mouse_state.last_pos != pos {
            self.mouse_state.moved = true;
        }
        self.mouse_state.last_pos = pos;
        match self.mouse_state.op {
            MouseDragOp::Panning { original_ul, start } => {
                let (x1, y1) = start;
                let (x2, y2) = self.mouse_state.last_pos;
                let (dx, dy) = (x2 - x1, y2 - y1);
                let xoff = original_ul.0 - dx as f32;
                let yoff = original_ul.1 - dy as f32;
                self.vstate.ul_offset = (xoff, yoff);
            }
            MouseDragOp::Select { start } => {
                let drag_end = self.byte_from_coords(self.mouse_state.last_pos);
                let drag_start = self.byte_from_coords(start);
                match (drag_start, drag_end) {
                    (Some(s), Some(e)) => self.set_selection(s, e),
                    _ => {}
                };
            }
            _ => {}
        }
    }

    fn byte_from_coords(&self, pos: (f64, f64)) -> Option<u32> {
        // find (possibly off-screen) location of 0,0 in data.
        // adjust for zoom
        let (x, y) = (
            (pos.0 + self.vstate.ul_offset.0 as f64) / self.vstate.zoom as f64,
            (pos.1 + self.vstate.ul_offset.1 as f64) / self.vstate.zoom as f64,
        );
        // add deltas to upper left corner of image

        if x < 0.0 || y < 0.0 || y >= self.dview.column_dim.1 as f64 {
            None
        } else {
            let cw = self.dview.column_dim.0 + self.dview.column_spacing;
            let column = x as u32 / cw;
            let row = y as u32;
            let el_in_row = x as u32 % cw;

            let el_per_b = (8 / self.dview.bits_per_pixel) as u32;
            let cw_in_b = self.dview.column_dim.0 / (self.dview.bits_per_pixel as u32);

            let el_idx = (cw_in_b * self.dview.column_dim.1 * column) + (row * cw_in_b) + el_in_row;
            let idx = el_idx / el_per_b;

            if idx < self.dview.data_len() as u32 {
                Some(idx)
            } else {
                None
            }
        }
    }

    fn handle_mouse_button(
        &mut self,
        button: glfw::MouseButton,
        action: glfw::Action,
        modifiers: glfw::Modifiers,
    ) {
        match action {
            glfw::Action::Press => {
                self.mouse_state.moved = false;
                self.mouse_state.last_pos = self.window.get_cursor_pos();
                self.mouse_state.op = match (button, modifiers) {
                    (glfw::MouseButtonLeft, glfw::Modifiers::Shift)
                    | (glfw::MouseButtonMiddle, _) => MouseDragOp::Panning {
                        original_ul: self.vstate.ul_offset,
                        start: self.mouse_state.last_pos,
                    },
                    (glfw::MouseButtonLeft, _) => MouseDragOp::Select {
                        start: self.mouse_state.last_pos,
                    },
                    _ => MouseDragOp::NoOp,
                };
            }
            glfw::Action::Release => {
                match self.mouse_state.op {
                    MouseDragOp::Select { start } if !self.mouse_state.moved => {
                        self.set_selection(0, 0);
                    }
                    _ => {}
                }
                self.mouse_state.op = MouseDragOp::NoOp;
            }
            _ => {}
        };
    }

    pub fn handle_events(&mut self) {
        use glfw::Action;
        loop {
            match self.events.try_recv() {
                Ok((_, event)) => match event {
                    glfw::WindowEvent::Key(key, _, Action::Press, _) => self.handle_kb(key),
                    glfw::WindowEvent::Key(key, _, Action::Repeat, _) => self.handle_kb(key),
                    glfw::WindowEvent::MouseButton(b, a, m) => self.handle_mouse_button(b, a, m),
                    glfw::WindowEvent::CursorPos(x, y) => self.handle_mouse_move((x, y)),
                    glfw::WindowEvent::Scroll(_, ydelta) => self.handle_scroll(ydelta),
                    glfw::WindowEvent::Size(x, y) => {
                        //self.col_height = y as u32;
                        unsafe {
                            gl::Viewport(0, 0, x, y);
                        }
                    }

                    _ => {}
                },
                _ => {
                    break;
                }
            }
        }
    }
}
