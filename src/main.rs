extern crate clap;
extern crate memmap;
#[macro_use]
extern crate glium;
extern crate glfw;
extern crate gl;

use clap::{Arg,App};

use memmap::{Mmap, Protection};

use std::str;

mod annotation;
mod viz;
mod font;
mod glutil;

use glfw::{Action, Context, Key};

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


    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let mut viz = viz::Visualizer::new(&mut glfw, (512, 512), unsafe { rom.as_slice() });
    viz.set_selection(800,1600);

    viz.window.make_current();
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
    while !viz.window.should_close() {
        viz.render();
        glfw.poll_events();
        viz.handle_events();
    }
}

