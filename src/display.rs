use crate::contracts::DisplayOutput;
use bitvec::prelude::*;
use getset::{Setters};
use sdl2::{
    Sdl,
    video::{Window, FullscreenType},
    render::Canvas,
    pixels::Color,
    rect::Rect,
    mouse::MouseUtil,
};

#[derive(Setters)]
pub struct WindowDisplay {
    canvas: Canvas<Window>,
    #[getset(set = "pub")]
    bg_color: Color,
    #[getset(set = "pub")]
    fg_color: Color,
    mouse: MouseUtil,
}

impl DisplayOutput for WindowDisplay {
    fn draw(&mut self, buffer: &BitArray<Msb0, [u64; 32]>) -> Result<(), String> {
        // Clear
        self.canvas.set_draw_color(self.bg_color);
        self.canvas.clear();

        // Draw frame
        let (width, height) = self.canvas.output_size()?;
        let scale_w = width / WindowDisplay::C8_WIDTH;
        let offset_x = (width - WindowDisplay::C8_WIDTH * scale_w) as i32 / 2;
        let scale_h = height / WindowDisplay::C8_HEIGHT;
        let offset_y = (height - WindowDisplay::C8_HEIGHT * scale_h) as i32 / 2;
        for y in 0..32 {
            for x in 0..64 {
                self.canvas.set_draw_color(if buffer[self.get_index(x, y)] { self.fg_color } else { self.bg_color });
                self.canvas.fill_rect(Rect::new(offset_x + scale_w as i32 * x as i32, offset_y + scale_h as i32 * y as i32, scale_w, scale_h))?;
            }
        }

        self.canvas.present();
        Ok(())
    }

    fn toggle_fullscreen(&mut self) -> Result<(), String> {
        let state = if self.canvas.window().fullscreen_state() == FullscreenType::Desktop { FullscreenType::Off } else { FullscreenType::Desktop };
        self.canvas.window_mut().set_fullscreen(state)?;
        self.mouse.show_cursor(state == FullscreenType::Off);
        Ok(())
    }
}

impl WindowDisplay {
    const WINDOW_NAME: &'static str = "pich8";
    const BG_COLOR: Color = Color::BLACK;
    const FG_COLOR: Color = Color::WHITE;
    const WINDOW_WIDTH: u32 = 800;
    const WINDOW_HEIGHT: u32 = WindowDisplay::WINDOW_WIDTH / (WindowDisplay::C8_WIDTH / WindowDisplay::C8_HEIGHT);
    const C8_WIDTH: u32 = 64;
    const C8_HEIGHT: u32 = 32;

    pub fn new(sdl_context: &Sdl, vsync: bool) -> Result<Self, String> {
        let window = sdl_context.video()?
            .window(WindowDisplay::WINDOW_NAME, WindowDisplay::WINDOW_WIDTH, WindowDisplay::WINDOW_HEIGHT)
            .position_centered()
            .resizable()
            .opengl()
            .build().map_err(|e| format!("couldn't setup window: {}", e))?;
        let mut canvas_builder = window.into_canvas();
        if vsync {
            canvas_builder = canvas_builder.present_vsync();
        }
        let canvas = canvas_builder.build().map_err(|e| format!("couldn't setup canvas: {}", e))?;

        Ok(Self{
            canvas,
            bg_color: WindowDisplay::BG_COLOR,
            fg_color: WindowDisplay::FG_COLOR,
            mouse: sdl_context.mouse(),
        })
    }

    fn get_index(&self, x: usize, y: usize) -> usize {
        (y * 64) + x
    }
}