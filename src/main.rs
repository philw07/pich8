#![cfg_attr(not(any(test, debug_assertions)), windows_subsystem = "windows")]

mod cpu;
mod display;
mod gui;
mod sound;
mod emulator;
mod serde_big_array;
mod file_dialog_handler;
mod fps_counter;

use emulator::Emulator;

fn main() {
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let mut emu = Emulator::new(&event_loop).unwrap();
    event_loop.run(move |event, _, ctrl_flow| emu.handle_event(event, ctrl_flow));
}
