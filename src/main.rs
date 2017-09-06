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
    
    // let (mut window, events) = glfw.create_window(300, 300, "Hello this is window 1", glfw::WindowMode::Windowed)
    //     .expect("Failed to create GLFW window.");

    // let (mut window2, events2) = glfw.create_window(300, 300, "Hello this is window 2", glfw::WindowMode::Windowed)
    //     .expect("Failed to create GLFW window.");

    // window.set_key_polling(true);
    // window2.set_key_polling(true);
    // window.make_current();
    
    while true {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&viz.events) {
            handle_window_event(&mut viz.window, event);
        }
    }
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            println!("got escape");
            window.set_should_close(true)
        }
        _ => {}
    }
}    
    // let mut viz = viz::Visualizer::new((512, 512), unsafe { rom.as_slice() });
    // viz.set_stride(8);
    // viz.set_selection(800,1600);
    // while !viz.closed {
    //     viz.render();
    //     viz.handle_events();
    // }

