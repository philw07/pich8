use std::time::Duration;
use glium::{Display, Surface, glutin::event::Event};
use getset::{CopyGetters, Getters, Setters};
use imgui::{Context, MenuItem, im_str, FontSource, FontId, Ui, ColorEdit};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

#[derive(CopyGetters, Getters, Setters)]
pub struct GUI {
    imgui: Context,
    renderer: Renderer,
    platform: WinitPlatform,
    custom_font: FontId,

    // For convenience we're only reporting the height of the last frame
    last_menu_height: u32,
    
    #[getset(get_copy = "pub")]
    menu_open: bool,

    // Flags
    #[getset(get_copy = "pub", set = "pub")]
    flag_open_rom: bool,
    #[getset(get_copy = "pub", set = "pub")]
    flag_load_state: bool,
    #[getset(get_copy = "pub", set = "pub")]
    flag_save_state: bool,
    #[getset(get_copy = "pub", set = "pub")]
    flag_reset: bool,
    #[getset(get_copy = "pub", set = "pub")]
    flag_exit: bool,

    #[getset(get_copy = "pub", set = "pub")]
    flag_fullscreen: bool,
    #[getset(get = "pub", set = "pub")]
    bg_color: [f32; 3],
    #[getset(get = "pub", set = "pub")]
    fg_color: [f32; 3],
    flag_display_fps: bool,

    #[getset(get_copy = "pub", set = "pub")]
    flag_pause: bool,
    #[getset(get_copy = "pub", set = "pub")]
    cpu_speed: u16,

    #[getset(get_copy = "pub", set = "pub")]
    flag_quirk_load_store: bool,
    #[getset(get_copy = "pub", set = "pub")]
    flag_quirk_shift: bool,
    #[getset(get_copy = "pub", set = "pub")]
    flag_vertical_wrapping: bool,
    #[getset(get_copy = "pub", set = "pub")]
    flag_mute: bool,
}

impl GUI {
    const FONT_SIZE: f32 = 16.0;
    const MENU_HEIGHT_CLEARANCE: u32 = 1;

    pub fn new(display: &Display) -> Self {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        // Load custom font
        let roboto = imgui.fonts().add_font(&[FontSource::TtfData {
            data: include_bytes!("../fonts/Roboto/Roboto-Regular.ttf"),
            size_pixels: GUI::FONT_SIZE,
            config: None,
        }]);
        
        // Create renderer and platform
        let renderer = Renderer::init(&mut imgui, display).expect("Failed to initialize renderer");
        let mut platform = WinitPlatform::init(&mut imgui);
        {
            let gl_win = display.gl_window();
            let window = gl_win.window();
            platform.attach_window(imgui.io_mut(), window, HiDpiMode::Rounded);
        }

        Self {
            imgui,
            renderer,
            platform,
            custom_font: roboto,
            last_menu_height: 0,
            menu_open: false,

            flag_open_rom: false,
            flag_load_state: false,
            flag_save_state: false,
            flag_reset: false,
            flag_exit: false,

            flag_fullscreen: false,
            bg_color: [0.0; 3],
            fg_color: [0.0; 3],
            flag_display_fps: false,

            flag_pause: false,

            cpu_speed: 0,

            flag_quirk_load_store: false,
            flag_quirk_shift: false,
            flag_vertical_wrapping: false,
            flag_mute: false,
        }
    }

    pub fn handle_event<T>(&mut self, display: &Display, event: &Event<T>) {
        let gl_window = display.gl_window();
        self.platform.handle_event(self.imgui.io_mut(), gl_window.window(), &event);
    }

    pub fn render<S: Surface>(&mut self, delta_time: Duration, display: &Display, target: &mut S) -> Result<(), String> {
        self.menu_open = false;
        self.imgui.io_mut().update_delta_time(delta_time);

        let ui = self.imgui.frame();
        let roboto = ui.push_font(self.custom_font);
        if let Some(menu_bar) = ui.begin_main_menu_bar() {
            if let Some(menu) = ui.begin_menu(im_str!("File"), true) {
                self.menu_open = true;
                MenuItem::new(im_str!("Open ROM..."))
                    .build_with_ref(&ui, &mut self.flag_open_rom);
                ui.separator();
                MenuItem::new(im_str!("Load State..."))
                    .build_with_ref(&ui, &mut self.flag_load_state);
                MenuItem::new(im_str!("Save State..."))
                    .build_with_ref(&ui, &mut self.flag_save_state);
                ui.separator();
                MenuItem::new(im_str!("Reset"))
                    .build_with_ref(&ui, &mut self.flag_reset);
                    ui.separator();
                MenuItem::new(im_str!("Exit"))
                    .build_with_ref(&ui, &mut self.flag_exit);
                menu.end(&ui);
            }
            if let Some(menu) = ui.begin_menu(im_str!("View"), true) {
                self.menu_open = true;
                MenuItem::new(im_str!("Fullscreen"))
                    .shortcut(im_str!("F11"))
                    .build_with_ref(&ui, &mut self.flag_fullscreen);
                ui.separator();
                ColorEdit::new(im_str!("Background Color"), &mut self.bg_color).build(&ui);
                ColorEdit::new(im_str!("Foreground Color"), &mut self.fg_color).build(&ui);
                ui.separator();
                MenuItem::new(im_str!("Display FPS"))
                    .build_with_ref(&ui, &mut self.flag_display_fps);
                menu.end(&ui);
            }
            if let Some(menu) = ui.begin_menu(im_str!("Settings"), true) {
                self.menu_open = true;
                MenuItem::new(im_str!("Pause"))
                    .shortcut(im_str!("P"))
                    .build_with_ref(&ui, &mut self.flag_pause);
                ui.separator();
                if let Some(cpu_speed_menu) = ui.begin_menu(im_str!("CPU Speed"), true) {
                    GUI::cpu_speed_menu_item(&ui, "Slowest", 420, &mut self.cpu_speed);
                    GUI::cpu_speed_menu_item(&ui, "Slow", 600, &mut self.cpu_speed);
                    GUI::cpu_speed_menu_item(&ui, "Normal", 720, &mut self.cpu_speed);
                    GUI::cpu_speed_menu_item(&ui, "Fast", 900, &mut self.cpu_speed);
                    GUI::cpu_speed_menu_item(&ui, "Faster", 1200, &mut self.cpu_speed);
                    GUI::cpu_speed_menu_item(&ui, "Fastest", 1500, &mut self.cpu_speed);
                    cpu_speed_menu.end(&ui);
                }
                if let Some(quirks_menu) = ui.begin_menu(im_str!("Quirks"), true) {
                    MenuItem::new(im_str!("Load/Store"))
                        .build_with_ref(&ui, &mut self.flag_quirk_load_store);
                    MenuItem::new(im_str!("Shift"))
                        .build_with_ref(&ui, &mut self.flag_quirk_shift);
                    quirks_menu.end(&ui);
                }
                MenuItem::new(im_str!("Vertical Wrapping"))
                    .build_with_ref(&ui, &mut self.flag_vertical_wrapping);
                ui.separator();
                MenuItem::new(im_str!("Mute Audio"))
                    .shortcut(im_str!("M"))
                    .build_with_ref(&ui, &mut self.flag_mute);
                menu.end(&ui);
            }

            if self.flag_display_fps {
                let fps = im_str!("{:.1} fps", ui.frame_count() as f64 / ui.time());
                let winwidth = display.gl_window().window().inner_size().width;
                let textwidth = ui.calc_text_size(&fps, false, 0.0);
                ui.same_line(winwidth as f32 - (textwidth[0] * 1.25));
                ui.text_colored([0.75, 0.75, 0.75, 1.0], fps);
            }

            // Store menu bar height with a bit of clearance
            self.last_menu_height = ui.window_size()[1] as u32 + GUI::MENU_HEIGHT_CLEARANCE;

            roboto.pop(&ui);
            menu_bar.end(&ui);
        }

        let gl_window = display.gl_window();
        self.platform.prepare_render(&ui, gl_window.window());

        let draw_data = ui.render();
        self.renderer.render(target, draw_data)
            .map_err(|e| format!("Failed to render UI: {}", e))?;

        Ok(())
    }

    pub fn menu_height(&self) -> u32 {
        self.last_menu_height
    }

    fn cpu_speed_menu_item(ui: &Ui, name: &str, item_speed: u16, current_speed: &mut u16) {
        let mut flag = *current_speed == item_speed;
        MenuItem::new(&im_str!("{} ({}Hz)", name, item_speed))
            .build_with_ref(ui, &mut flag);
        if flag { *current_speed = item_speed; }
    }
}
