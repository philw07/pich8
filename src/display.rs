use crate::video_memory::{VideoMemory, Plane};
use getset::{Getters, Setters};
use glium::{
    Display,
    glutin::{
        window::{WindowBuilder, Icon},
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
    width: u32,
    height: u32,
    #[getset(get = "pub", set = "pub")]
    color_bg: [u8; 3],
    #[getset(get = "pub", set = "pub")]
    color_plane_1: [u8; 3],
    #[getset(get = "pub", set = "pub")]
    color_plane_2: [u8; 3],
    #[getset(get = "pub", set = "pub")]
    color_plane_both: [u8; 3],
}

impl WindowDisplay {
    const WINDOW_TITLE: &'static str = "pich8";
    const WINDOW_WIDTH: f32 = 800.0;
    const WINDOW_HEIGHT: f32 = WindowDisplay::WINDOW_WIDTH / (WindowDisplay::C8_WIDTH as f32 / WindowDisplay::C8_HEIGHT as f32);
    const C8_WIDTH: usize = 64;
    const C8_HEIGHT: usize = 32;

    pub fn new(event_loop: &EventLoop<()>, vsync: bool) -> Result<Self, String> {
        // Load icon
        let icon_file = include_bytes!("../data/icon/pich8_32.png");
        let icon_image = image::load_from_memory_with_format(icon_file, image::ImageFormat::Png).map_err(|e| format!("Failed to load icon: {}", e))?.into_rgba8();
        let (width, height) = icon_image.dimensions();
        let icon = Icon::from_rgba(icon_image.into_raw(), width, height).map_err(|e| format!("Failed to parse icon: {}", e))?;

        // Create window
        let context = ContextBuilder::new().with_vsync(vsync);
        let builder = WindowBuilder::new()
            .with_window_icon(Some(icon))
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
            if let Some(monitor_handle) = window.current_monitor() {
                let monitor_size = monitor_handle.size();
                let window_size = window.outer_size();
                let position = glium::glutin::dpi::PhysicalPosition::new(
                    monitor_size.width / 2 - window_size.width / 2,
                    monitor_size.height / 2 - window_size.height / 2,
                );
                display.gl_window().window().set_outer_position(position);
            }
        }
        
        // Clear screen with bg color
        let mut target = display.draw();
        let color_bg = [0; 3];
        target.clear_color(color_bg[0] as f32 / 255.0, color_bg[1] as f32 / 255.0, color_bg[2] as f32 / 255.0, 1.0);
        target.finish().map_err(|e| format!("Failed to swap buffers: {}", e))?;

        Ok(Self{
            display,
            frame_buffer: [0; 2*Self::C8_WIDTH * 2*Self::C8_HEIGHT * 3],
            width: 0,
            height: 0,
            color_bg,
            color_plane_1: [0; 3],
            color_plane_2: [0; 3],
            color_plane_both: [0; 3],
        })
    }

    pub fn display(&self) -> &Display {
        &self.display
    }

    fn copy_frame(&mut self, vmem: &VideoMemory) {
        for idx in 0..vmem.render_width()*vmem.render_height() {
            let buf_idx = idx * 3;
            if vmem.get_index_plane(Plane::First, idx) && vmem.get_index_plane(Plane::Second, idx) {
                self.frame_buffer[buf_idx..buf_idx+3].copy_from_slice(&self.color_plane_both);
            } else if vmem.get_index_plane(Plane::First, idx) {
                self.frame_buffer[buf_idx..buf_idx+3].copy_from_slice(&self.color_plane_1);
            } else if vmem.get_index_plane(Plane::Second, idx) {
                self.frame_buffer[buf_idx..buf_idx+3].copy_from_slice(&self.color_plane_2);
            } else {
                self.frame_buffer[buf_idx..buf_idx+3].copy_from_slice(&self.color_bg);
            }
        }
        self.width = vmem.render_width() as u32;
        self.height = vmem.render_height() as u32;
    }

    pub fn prepare(&mut self, vmem: Option<&VideoMemory>, menu_height: u32) -> Result<Frame, String> {
        // Copy over new frame
        if let Some(vmem) = vmem {
            self.copy_frame(vmem);
        }
        let frame_len = self.width as usize * self.height as usize * 3;

        // Prepare texture
        let mut frame = self.display.draw();
        frame.clear_color(self.color_bg[0] as f32 / 255.0, self.color_bg[1] as f32 / 255.0, self.color_bg[2] as f32 / 255.0, 1.0);
        let img = RawImage2d::from_raw_rgb_reversed(&self.frame_buffer[..frame_len], (self.width, self.height));
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
