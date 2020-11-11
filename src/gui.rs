use std::time::Duration;
use glium::{Display, Surface, glutin::event::Event};
use getset::{CopyGetters, Getters, Setters};
use imgui::{Context, MenuItem, im_str, FontSource, FontId, Ui, ColorEdit, Window, Condition, ImString};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

#[derive(CopyGetters, Getters, Setters)]
pub struct GUI {
    imgui: Context,
    renderer: Renderer,
    platform: WinitPlatform,
    custom_font: FontId,
    custom_font_big: FontId,

    // For convenience we're only reporting the height of the last frame
    last_menu_height: u32,
    
    #[getset(get_copy = "pub")]
    is_open: bool,

    // Flags
    #[getset(get_copy = "pub", set = "pub")]
    flag_open: bool,
    #[getset(get_copy = "pub", set = "pub")]
    flag_open_rom_url: bool,
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
    flag_quirk_draw: bool,
    #[getset(get_copy = "pub", set = "pub")]
    flag_vertical_wrapping: bool,
    #[getset(get_copy = "pub", set = "pub")]
    flag_mute: bool,

    flag_about: bool,
    flag_error: bool,
    error_text: ImString,
    #[getset(get_copy = "pub", set = "pub")]
    flag_downloading: bool,
}

impl GUI {
    const FONT_SIZE: f32 = 16.0;
    const MENU_HEIGHT_CLEARANCE: u32 = 1;

    pub fn new(display: &Display) -> Self {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        // Load custom font
        let roboto_data = include_bytes!("../data/fonts/Roboto/Roboto-Regular.ttf");
        let roboto = imgui.fonts().add_font(&[FontSource::TtfData {
            data: roboto_data,
            size_pixels: GUI::FONT_SIZE,
            config: None,
        }]);
        let roboto_big = imgui.fonts().add_font(&[FontSource::TtfData {
            data: roboto_data,
            size_pixels: GUI::FONT_SIZE + 4.0,
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
            custom_font_big: roboto_big,
            last_menu_height: 0,
            is_open: false,

            flag_open: false,
            flag_open_rom_url: false,
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
            flag_quirk_draw: false,
            flag_vertical_wrapping: false,
            flag_mute: false,

            flag_about: false,
            flag_error: false,
            error_text: ImString::new(""),
            flag_downloading: false,
        }
    }

    pub fn handle_event<T>(&mut self, display: &Display, event: &Event<T>) {
        let gl_window = display.gl_window();
        self.platform.handle_event(self.imgui.io_mut(), gl_window.window(), &event);
    }

    pub fn render<S: Surface>(&mut self, delta_time: Duration, display: &Display, target: &mut S, fps: f64) -> Result<(), String> {
        self.is_open = false;
        self.imgui.io_mut().update_delta_time(delta_time);

        let window_width = display.gl_window().window().inner_size().width;
        let window_height = display.gl_window().window().inner_size().height;

        let ui = self.imgui.frame();
        let cfont = ui.push_font(self.custom_font);
        if let Some(menu_bar) = ui.begin_main_menu_bar() {
            if let Some(menu) = ui.begin_menu(im_str!("File"), true) {
                self.is_open = true;
                MenuItem::new(im_str!("Open ROM or State..."))
                    .shortcut(im_str!("Ctrl + O"))
                    .build_with_ref(&ui, &mut self.flag_open);
                MenuItem::new(im_str!("Open ROM from URL..."))
                    .shortcut(im_str!("Ctrl + Shift + O"))
                    .build_with_ref(&ui, &mut self.flag_open_rom_url);
                MenuItem::new(im_str!("Save State..."))
                .shortcut(im_str!("Ctrl + S"))
                    .build_with_ref(&ui, &mut self.flag_save_state);
                ui.separator();
                MenuItem::new(im_str!("Reset"))
                .shortcut(im_str!("F5"))
                    .build_with_ref(&ui, &mut self.flag_reset);
                    ui.separator();
                MenuItem::new(im_str!("Exit"))
                    .shortcut(im_str!("Esc"))
                    .build_with_ref(&ui, &mut self.flag_exit);
                menu.end(&ui);
            }
            if let Some(menu) = ui.begin_menu(im_str!("View"), true) {
                self.is_open = true;
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
                self.is_open = true;
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
                    MenuItem::new(im_str!("Draw"))
                        .build_with_ref(&ui, &mut self.flag_quirk_draw);
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
            if let Some(menu) = ui.begin_menu(im_str!("Help"), true) {
                self.is_open = true;
                MenuItem::new(im_str!("About"))
                    .build_with_ref(&ui, &mut self.flag_about);
                menu.end(&ui);
            }

            if self.flag_display_fps {
                let fps = im_str!("{:.0} fps", fps);
                let text_width = ui.calc_text_size(&fps, false, 0.0);
                ui.same_line(window_width as f32 - (text_width[0] * 1.25));
                ui.text_colored([0.75, 0.75, 0.75, 1.0], fps);
            }
            if self.flag_downloading {
                self.is_open = true;
                let text = im_str!("Downloading...");
                let text_size = ui.calc_text_size(text, false, 250.0);
                let dl_win_size = [text_size[0] + 50.0, text_size[1] + 40.0];
                let dl_win_pos = [
                    window_width as f32 / 2.0 - dl_win_size[0] / 2.0,
                    window_height as f32 / 2.0 - dl_win_size[1] / 2.0
                ];
                Window::new(im_str!("Downloading"))
                    .position(dl_win_pos, Condition::Always)
                    .size(dl_win_size, Condition::Always)
                    .resizable(false)
                    .collapsible(false)
                    .movable(false)
                    .title_bar(false)
                    .build(&ui, || {
                        ui.set_cursor_pos([dl_win_size[0] / 2.0 - text_size[0] / 2.0, dl_win_size[1] / 2.0 - text_size[1] / 2.0]);
                        ui.text_wrapped(&text);
                    });
            }
            if self.flag_about {
                self.is_open = true;
                let about_win_size = [250.0, 110.0];
                let about_win_pos = [
                    window_width as f32 / 2.0 - about_win_size[0] / 2.0,
                    window_height as f32 / 2.0 - about_win_size[1] / 2.0
                ];
                let custom_font_big = self.custom_font_big;
                Window::new(im_str!("About"))
                    .opened(&mut self.flag_about)
                    .position(about_win_pos, Condition::Always)
                    .size(about_win_size, Condition::Always)
                    .resizable(false)
                    .collapsible(false)
                    .movable(false)
                    .build(&ui, || {
                        let cfont_big = ui.push_font(custom_font_big);
                        GUI::centered_text(&ui, im_str!("pich8"), about_win_size[0]);
                        cfont_big.pop(&ui);

                        ui.spacing();
                        GUI::centered_text(&ui, im_str!("A cross-platform CHIP-8"), about_win_size[0]);
                        GUI::centered_text(&ui, im_str!("interpreter written in Rust"), about_win_size[0]);
                    });
            }
            if self.flag_error {
                self.is_open = true;
                let text_size = ui.calc_text_size(&self.error_text, false, 250.0);
                let error_win_size = [text_size[0] + 50.0, text_size[1] + 40.0];
                let error_win_pos = [
                    window_width as f32 / 2.0 - error_win_size[0] / 2.0,
                    window_height as f32 / 2.0 - error_win_size[1] / 2.0
                ];
                let error_text = &self.error_text;
                Window::new(im_str!("Error"))
                .opened(&mut self.flag_error)
                .position(error_win_pos, Condition::Always)
                .size(error_win_size, Condition::Always)
                .resizable(false)
                .collapsible(false)
                .movable(false)
                .build(&ui, || {
                    ui.set_cursor_pos([error_win_size[0] / 2.0 - text_size[0] / 2.0, ui.cursor_pos()[1]]);
                    ui.text_wrapped(&error_text);
                });
            }

            // Store menu bar height with a bit of clearance
            self.last_menu_height = ui.window_size()[1] as u32 + GUI::MENU_HEIGHT_CLEARANCE;

            cfont.pop(&ui);
            menu_bar.end(&ui);
        }

        let gl_window = display.gl_window();
        self.platform.prepare_render(&ui, gl_window.window());

        let draw_data = ui.render();
        self.renderer.render(target, draw_data)
            .map_err(|e| format!("Failed to render UI: {}", e))?;

        Ok(())
    }

    fn centered_text(ui: &Ui, text: &imgui::ImStr, window_width: f32) {
        let text_width = ui.calc_text_size(text, false, 0.0)[0];
        ui.set_cursor_pos([window_width / 2.0 - text_width / 2.0, ui.cursor_pos()[1]]);
        ui.text_wrapped(&text);
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

    pub fn display_error(&mut self, message: &str) {
        self.flag_error = true;
        self.error_text = ImString::new(message);
    }
}
