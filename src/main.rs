extern crate clap;
extern crate memmap;
extern crate glfw;
extern crate gl;

use clap::{Arg,App};

use memmap::{Mmap, Protection};

use std::str;

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
    viz.set_stride(8);
    viz.set_selection(800,1600);
    viz.set_zoom(1.0);
    while !viz.win.should_close() {
        viz.render();
        viz.glfw.poll_events();
        viz.handle_events();
    }

}
