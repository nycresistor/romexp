extern crate clap;
extern crate memmap;
extern crate glfw;
extern crate gl;

use clap::{Arg,App};

use memmap::{Mmap, Protection};

use glfw::{Action, Context, Key};
use gl::types::*;
use std::mem;
use std::ptr;
use std::str;
use std::ffi::CString;

mod viz;

        

fn main() {
    let matches = App::new("ROM image explorer")
        .version("0.1")
        .author("phooky@gmail.com")
        .about("Quickly analyze ROM dumps and other binary blobs")
        .arg(Arg::with_name("ROM")
            .help("ROM file to analyze")
            .required(true))
        .get_matches();

    let rom_path = matches.value_of("ROM").unwrap();
    let rom = match Mmap::open_path(rom_path,Protection::Read) {
        Ok(r) => r,
        Err(e) => { println!("Could not open {}: {}",rom_path,e); return; },
    };
    
    println!("Opened {}; size {} bytes",rom_path,rom.len());

    let mut viz = viz::Visualizer::new((512, 512), rom.len());
    unsafe { viz.set_data(rom.as_slice()); }
    let mut stride = 8;
    let mut zoom = 1.0;
    viz.set_stride(stride);
    viz.set_selection(800,1600);
    viz.set_zoom(zoom);
    while !viz.win.should_close() {
        unsafe { gl::ClearColor(1.0,0.0,0.0,1.0) };
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT) };
        unsafe { 
            gl::BindTexture(gl::TEXTURE_1D, 1 );
            gl::DrawArrays(gl::TRIANGLES, 0, 6) };
        viz.win.swap_buffers();
        viz.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&viz.events) {
            match event {
                glfw::WindowEvent::Key(Key::Right, _, Action::Press, _) |
                glfw::WindowEvent::Key(Key::Right, _, Action::Repeat, _) => {
                    if stride < (rom.len() - 7) as u32 {
                        stride = stride + 8;
                        viz.set_stride(stride);
                    }
                },
                glfw::WindowEvent::Key(Key::Left, _, Action::Press, _) |
                glfw::WindowEvent::Key(Key::Left, _, Action::Repeat, _)=> {
                    if stride > 8 {
                        stride = stride - 8;
                        viz.set_stride(stride);
                    }
                },
                glfw::WindowEvent::Key(Key::Down, _, Action::Press, _) => {
                    if zoom <= 1.0 { zoom = zoom / 2.0; }
                    else { zoom = zoom - 1.0; }
                    
                    viz.set_zoom(zoom);
                },
                glfw::WindowEvent::Key(Key::Up, _, Action::Press, _) => {
                    if zoom >= 1.0 { zoom = zoom + 1.0; }
                    else { zoom = zoom * 2.0; }
                    viz.set_zoom(zoom);
                },
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    viz.win.set_should_close(true)
                },
                _ => {},
            }
        }
    }

}
