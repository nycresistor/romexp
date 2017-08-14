extern crate clap;
extern crate memmap;
#[macro_use]
extern crate glium;
#[macro_use]
extern crate imgui;
extern crate imgui_glium_renderer;

use clap::{Arg,App};

use memmap::{Mmap, Protection};

use std::str;

mod viz;

use glium::glutin;

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

    let mut events_loop = glutin::EventsLoop::new();

    println!("Opened {}; size {} bytes",rom_path,rom.len());
    let mut viz = viz::Visualizer::new((512, 512), &mut events_loop, unsafe { rom.as_slice() });
    viz.set_stride(8);
    viz.set_selection(800,1600);
    while !viz.closed {
        viz.render();
        viz.handle_events(&mut events_loop);
    }

}
