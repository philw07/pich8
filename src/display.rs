use crate::contracts::DisplayOutput;
use bitvec::prelude::*;
use getset::{Setters};
use sdl2::{
    Sdl,
    video::Window,
    render::Canvas,
    pixels::Color,
    rect::Rect,
};

#[derive(Setters)]
pub struct WindowDisplay {
    canvas: Canvas<Window>,
    #[getset(set = "pub")]
    bg_color: Color,
    #[getset(set = "pub")]
    fg_color: Color,
}

impl DisplayOutput for WindowDisplay {
    fn draw(&mut self, buffer: &BitArray<Msb0, [u64; 32]>) -> Result<(), String> {
        self.canvas.set_draw_color(self.bg_color);
        self.canvas.clear();

        // Draw
        let scale = 10;
        for y in 0..32 {
            for x in 0..64 {
                self.canvas.set_draw_color(if buffer[self.get_index(x, y)] { self.fg_color } else { self.bg_color });
                self.canvas.fill_rect(Rect::new(scale*x as i32, scale*y as i32, scale as u32, scale as u32))?;
            }
        }

        self.canvas.present();
        Ok(())
    }
}

impl WindowDisplay {
    const WINDOW_NAME: &'static str = "pich8";
    const BG_COLOR: Color = Color::BLACK;
    const FG_COLOR: Color = Color::WHITE;

    pub fn new(sdl_context: &Sdl) -> Result<Self, String> {
        let window = sdl_context
            .video()?
            .window(WindowDisplay::WINDOW_NAME, 640, 320)
            .position_centered()
            .opengl()
            .build().map_err(|e| format!("couldn't setup window: {}", e))?;
        let canvas = window.into_canvas().build().map_err(|e| format!("couldn't setup canvas: {}", e))?;

        Ok(Self{
            canvas: canvas,
            bg_color: WindowDisplay::BG_COLOR,
            fg_color: WindowDisplay::FG_COLOR,
        })
    }

    fn get_index(&self, x: usize, y: usize) -> usize {
        (y * 64) + x
    }
}