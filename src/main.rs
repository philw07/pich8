#![cfg_attr(not(any(test, debug_assertions)), windows_subsystem = "windows")]

mod cpu;
mod dialog_handler;
mod display;
mod emulator;
mod fps_counter;
mod gui;
mod sound;
mod video_memory;

#[cfg(feature = "rom-download")]
mod rom_downloader;

use emulator::Emulator;
use getopts::Options;
use std::env;

const OPT_VSYNC: &str = "vsync";

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("", OPT_VSYNC, "Turn on vsync");

    let mut vsync = false;
    if let Ok(matches) = opts.parse(args) {
        vsync = matches.opt_present(OPT_VSYNC);
    }

    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let mut emu = Emulator::new(&event_loop, vsync).expect("Failed to create emulator");
    event_loop.run(move |event, _, ctrl_flow| emu.handle_event(event, ctrl_flow));
}
