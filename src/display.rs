use crate::video_memory::VideoMemory;
use getset::{Getters, Setters};
use glium::{
    Display,
    glutin::{
        window::WindowBuilder,
        event_loop::EventLoop,
        dpi::LogicalSize,
        ContextBuilder,
    },
    implement_vertex,
    Frame,
    Surface,
    uniforms::MagnifySamplerFilter,
    texture::{
        Texture2d,
        RawImage2d,
    },
};

#[derive(Getters, Setters)]
pub struct WindowDisplay {
    display: Display,
    frame_buffer: [u8; 2*WindowDisplay::C8_WIDTH * 2*WindowDisplay::C8_HEIGHT * 3],
    #[getset(get = "pub", set = "pub")]
    bg_color: [u8; 3],
    #[getset(get = "pub", set = "pub")]
    fg_color: [u8; 3],
}

impl WindowDisplay {
    const WINDOW_TITLE: &'static str = "pich8";
    const BG_COLOR: [u8; 3] = [0; 3];
    const FG_COLOR: [u8; 3] = [255; 3];
    const WINDOW_WIDTH: f32 = 800.0;
    const WINDOW_HEIGHT: f32 = WindowDisplay::WINDOW_WIDTH / (WindowDisplay::C8_WIDTH as f32 / WindowDisplay::C8_HEIGHT as f32);
    const C8_WIDTH: usize = 64;
    const C8_HEIGHT: usize = 32;

    pub fn new(event_loop: &EventLoop<()>, vsync: bool) -> Result<Self, String> {
        // Create window
        let context = ContextBuilder::new().with_vsync(vsync);
        let builder = WindowBuilder::new()
            .with_title(WindowDisplay::WINDOW_TITLE)
            .with_min_inner_size(LogicalSize::new(8.0 * WindowDisplay::C8_WIDTH as f32, 8.0 * WindowDisplay::C8_HEIGHT as f32))
            .with_inner_size(LogicalSize::new(WindowDisplay::WINDOW_WIDTH, WindowDisplay::WINDOW_HEIGHT));
        let display = Display::new(builder, context, event_loop)
            .map_err(|e| format!("Failed to create display: {}", e))?;

        {
            // Unfortunately, the position cannot be set before constructing the window.
            // At least on Windows that leads to the window "jumping" to the set position after creation.
            let gl_window = &display.gl_window();
            let window = gl_window.window();
            let monitor_size = window.current_monitor().size();
            let window_size = window.outer_size();
            let position = glium::glutin::dpi::PhysicalPosition::new(
                monitor_size.width / 2 - window_size.width / 2,
                monitor_size.height / 2 - window_size.height / 2,
            );
            display.gl_window().window().set_outer_position(position);
        }
        
        // Clear screen with bg color
        let mut target = display.draw();
        let bg_color = WindowDisplay::BG_COLOR;
        target.clear_color(bg_color[0] as f32 / 255.0, bg_color[1] as f32 / 255.0, bg_color[2] as f32 / 255.0, 1.0);
        target.finish().map_err(|e| format!("Failed to swap buffers: {}", e))?;

        Ok(Self{
            display,
            frame_buffer: [0; 2*WindowDisplay::C8_WIDTH * 2*WindowDisplay::C8_HEIGHT * 3],
            bg_color,
            fg_color: WindowDisplay::FG_COLOR,
        })
    }

    pub fn display(&self) -> &Display {
        &self.display
    }

    pub fn prepare(&mut self, vmem: &VideoMemory, menu_height: u32) -> Result<Frame, String> {
        // Copy over new frame
        for idx in 0..vmem.render_width()*vmem.render_height() {
            let buf_idx = idx * 3;
            if vmem[idx] {
                self.frame_buffer[buf_idx..buf_idx+3].copy_from_slice(&self.fg_color);
            } else {
                self.frame_buffer[buf_idx..buf_idx+3].copy_from_slice(&self.bg_color);
            }
        }
        let frame_len = vmem.render_width() * vmem.render_height() * 3;

        // Prepare texture
        let mut frame = self.display.draw();
        frame.clear_color(self.bg_color[0] as f32 / 255.0, self.bg_color[1] as f32 / 255.0, self.bg_color[2] as f32 / 255.0, 1.0);
        let img = RawImage2d::from_raw_rgb_reversed(&self.frame_buffer[..frame_len], (vmem.render_width() as u32, vmem.render_height() as u32));
        let texture = Texture2d::new(&self.display, img)
            .map_err(|e| format!("Failed to create texture: {}", e))?;

        let window_size = self.display.gl_window().window().inner_size();
        let height = window_size.height - menu_height;
        texture.as_surface().blit_whole_color_to(&frame,
            &glium::BlitTarget { left: 0, bottom: 0, width: window_size.width as i32, height: height as i32 }, MagnifySamplerFilter::Nearest);

        Ok(frame)
    }

    pub fn render(&self, frame: Frame) -> Result<(), String> {
        frame.finish()
            .map_err(|e| format!("Failed to swap buffers: {}", e))?;
        Ok(())
    }

    pub fn fullscreen(&self) -> bool {
        self.display.gl_window().window().fullscreen() != None
    }

    pub fn toggle_fullscreen(&mut self) -> Result<(), String> {
        let gl_window = self.display.gl_window();
        let monitor_handle = gl_window.window().current_monitor();
        let state = if gl_window.window().fullscreen().is_none() { Some(glium::glutin::window::Fullscreen::Borderless(monitor_handle)) } else { None };
        gl_window.window().set_cursor_visible(state == None);
        gl_window.window().set_fullscreen(state);
        Ok(())
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);
