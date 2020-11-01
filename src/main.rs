#![cfg_attr(not(any(test, debug_assertions)), windows_subsystem = "windows")]

mod contracts;
mod cpu;
mod display;
mod gui;
mod sound;
mod emulator;
mod serde_big_array;
mod file_dialog_handler;

use emulator::Emulator;

fn main() {
    let mut path = "./roms/c8games/BRIX".to_string();
    if std::env::args().len() > 1 {
        let v: Vec<String> = std::env::args().collect();
        path = v[1].clone();
    }
    
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let mut emu = Emulator::new_without_sound(&event_loop).unwrap();
    emu.load_rom(&std::fs::read(&path).unwrap());
    event_loop.run(move |event, _, ctrl_flow| emu.handle_event(event, ctrl_flow));
}
