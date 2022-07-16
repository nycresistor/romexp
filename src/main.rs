extern crate memmap;
extern crate glfw;
extern crate gl;
#[macro_use]
extern crate clap;

use clap::{Arg,App};

use memmap::{Mmap, Protection};

use std::str;
use std::cmp;

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
        .arg(Arg::with_name("wordsize")
             .help("Starting word size in bytes")
             .short('w')
             .long("wordsize")
             .takes_value(true)
             .default_value("1"))
        .arg(Arg::with_name("intercolumn")
             .help("space between columns")
             .long("intercolumn")
             .short('i')
             .takes_value(true)
             .default_value("0"))
        .arg(Arg::with_name("offset")
            .help("initial offset into binary blob")
            .short('o')
            .takes_value(true)
            .default_value("0"))
        .arg(Arg::with_name("ROM")
            .help("ROM file to analyze")
            .required(true))
        .get_matches();

    let rom_path = matches.value_of("ROM").unwrap();
    let rom = match Mmap::open_path(rom_path,Protection::Read) {
        Ok(r) => r,
        Err(e) => { println!("Could not open {}: {}",rom_path,e); return; },
    };
    let word = value_t_or_exit!(matches,"wordsize",u32) * 8;
    println!("Opened {}; size {} bytes",rom_path,rom.len());

    let height = 512;
    let spacing = value_t_or_exit!(matches,"intercolumn",u32); // default spacing in px
    let offset = value_t_or_exit!(matches,"offset",usize); // initial offset
    let bytes_per_column = (word/8)*height;
    let columns = rom.len() as u32 / bytes_per_column;
    let width = cmp::max(512,columns*(word+spacing));

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let mut viz = viz::Visualizer::new(&mut glfw, (width, height), unsafe { rom.as_slice() });
    viz.set_col_width(word);
    viz.set_spacing(spacing);
    viz.set_offset(offset);
    viz.window.make_current();
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
    while !viz.window.should_close() {
        viz.render();
        glfw.wait_events();
        viz.handle_events();
    }
}

