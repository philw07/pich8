use crate::contracts::DisplayOutput;
use bitvec::prelude::*;
use minifb::{Window, WindowOptions, Key};

pub struct WindowDisplay {
    window: Window,
}

impl DisplayOutput for WindowDisplay {
    fn draw(&mut self, buffer: &BitArray<Msb0, [u64; 32]>) {
        let mut big_buffer: [u32; 640*320] = [0; 640*320];
        for x in 0..640 {
            for y in 0..320 {
                let big_pos = self.get_coordinate(x, y, true);
                let small_pos = self.get_coordinate(x/10, y/10, false);
                if buffer[small_pos] {
                    big_buffer[big_pos] = 0x00FF00;
                }
            } 
        }
        self.window.update_with_buffer(&big_buffer, 640, 320).expect("couldn't update window");
    }
}

impl WindowDisplay {
    const WINDOW_NAME: &'static str = "c8e-rs";

    pub fn new() -> Self {
        Self{
            window: Window::new(WindowDisplay::WINDOW_NAME, 640, 320, WindowOptions::default()).expect("couldn't create window"),
        }
    }

    pub fn update(&mut self) {
        self.window.update();
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.window.is_key_down(key)
    }

    fn get_coordinate(&self, x: usize, y: usize, big: bool) -> usize {
        if big {
            (y * 640) + x
        } else {
            (y * 64) + x
        }
    }
}