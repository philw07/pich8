use crate::cpu::CPU;
use color_presets::{ColorPreset, ColorPresetHandler};
pub use color_settings::Color;
use color_settings::ColorSettings;
use glium::{glutin::event::Event, Display, Surface};
use imgui::{
    im_str, ColorEdit, Condition, Context, FontId, FontSource, ImStr, ImString, MenuItem, Slider,
    StyleColor, Ui, Window,
};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use quirks_presets::{QuirksPreset, QuirksPresetHandler};
pub use quirks_settings::Quirk;
use quirks_settings::QuirksSettings;
use std::time::Duration;

mod color_presets;
mod color_settings;
mod quirks_presets;
mod quirks_settings;

pub struct GUI {
    imgui: Context,
    renderer: Renderer,
    platform: WinitPlatform,
    custom_font: FontId,
    custom_font_big: FontId,
    custom_font_small: FontId,

    // For convenience we're only reporting the height of the last frame
    last_menu_height: u32,

    is_open: bool,

    // Flags
    pub flag_open: bool,

    #[cfg(feature = "rom-download")]
    pub flag_open_rom_url: bool,

    pub flag_save_state: bool,
    pub flag_reset: bool,
    pub flag_exit: bool,

    pub flag_fullscreen: bool,
    pub flag_display_fps: bool,
    pub flag_debug: bool,

    color_settings: ColorSettings,

    pub flag_pause: bool,
    pub cpu_speed: u32,
    cpu_multiplier: u32,
    pub flag_mute: bool,
    pub volume: f32,

    quirks_settings: QuirksSettings,

    flag_about: bool,
    flag_error: bool,
    error_text: ImString,
    pub flag_downloading: bool,
    pub flag_step: bool,
    pub flag_step_timers: bool,

    flag_breakpoint_pc: bool,
    breakpoint_pc_im: ImString,
    breakpoint_pc: String,
    flag_breakpoint_i: bool,
    breakpoint_i_im: ImString,
    breakpoint_i: String,
    flag_breakpoint_opcode: bool,
    breakpoint_opcode_im: ImString,
    breakpoint_opcode: String,

    about_name: ImString,
    about_version: ImString,
    about_description: ImString,
    about_license: ImString,
}

impl GUI {
    const FONT_SIZE: f32 = 16.0;
    const MENU_HEIGHT_CLEARANCE: u32 = 1;
    const WIDTH_TEXTBOX_REGISTER: f32 = 32.0;
    const COLOR_TEXT_DISABLED: [f32; 4] = [1.0, 1.0, 1.0, 0.5];

    pub fn new(display: &Display) -> Self {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        // Load custom font
        let roboto_data = include_bytes!("../../data/fonts/Roboto/Roboto-Regular.ttf");
        let roboto = imgui.fonts().add_font(&[FontSource::TtfData {
            data: roboto_data,
            size_pixels: Self::FONT_SIZE,
            config: None,
        }]);
        let roboto_big = imgui.fonts().add_font(&[FontSource::TtfData {
            data: roboto_data,
            size_pixels: Self::FONT_SIZE + 4.0,
            config: None,
        }]);
        let robotomono_data = include_bytes!("../../data/fonts/Roboto/RobotoMono-Regular.ttf");
        let roboto_small = imgui.fonts().add_font(&[FontSource::TtfData {
            data: robotomono_data,
            size_pixels: Self::FONT_SIZE - 3.0,
            config: None,
        }]);

        // Set default breakpoint values
        let mut breakpoint_pc_im = ImString::with_capacity(4);
        breakpoint_pc_im.push_str("0");
        let breakpoint_pc = String::from(breakpoint_pc_im.to_str());
        let mut breakpoint_i_im = ImString::with_capacity(4);
        breakpoint_i_im.push_str("0");
        let breakpoint_i = String::from(breakpoint_i_im.to_str());
        let mut breakpoint_opcode_im = ImString::with_capacity(4);
        breakpoint_opcode_im.push_str("****");
        let breakpoint_opcode = String::from(breakpoint_opcode_im.to_str());

        // Set default presets
        let mut color_settings = ColorSettings::new();
        ColorPresetHandler::new(&mut color_settings).set_preset(ColorPreset::Default);
        color_settings.changed = true;

        let mut quirks_settings = QuirksSettings::new();
        QuirksPresetHandler::new(&mut quirks_settings).set_preset(QuirksPreset::Default);

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
            custom_font_small: roboto_small,
            last_menu_height: 0,
            is_open: false,

            flag_open: false,

            #[cfg(feature = "rom-download")]
            flag_open_rom_url: false,

            flag_save_state: false,
            flag_reset: false,
            flag_exit: false,

            flag_fullscreen: false,
            color_settings,

            flag_display_fps: false,
            flag_debug: false,

            flag_pause: false,

            cpu_speed: 0,
            cpu_multiplier: 1,

            flag_mute: false,
            volume: 0.0,

            quirks_settings,

            flag_about: false,
            flag_error: false,
            error_text: ImString::new(""),
            flag_downloading: false,
            flag_step: false,
            flag_step_timers: false,

            flag_breakpoint_pc: false,
            breakpoint_pc_im,
            breakpoint_pc,
            flag_breakpoint_i: false,
            breakpoint_i_im,
            breakpoint_i,
            flag_breakpoint_opcode: false,
            breakpoint_opcode_im,
            breakpoint_opcode,

            about_name: ImString::from(env!("CARGO_PKG_NAME").to_string()),
            about_version: ImString::from(env!("CARGO_PKG_VERSION").to_string()),
            about_description: ImString::from(env!("CARGO_PKG_DESCRIPTION").to_string()),
            about_license: ImString::from(format!(
                "Released under the {} license",
                env!("CARGO_PKG_LICENSE").to_string()
            )),
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }
    pub fn color_settings(&mut self) -> &mut ColorSettings {
        &mut self.color_settings
    }
    pub fn quirks_settings(&self) -> &QuirksSettings {
        &self.quirks_settings
    }
    pub fn flag_breakpoint_pc(&self) -> bool {
        self.flag_breakpoint_pc
    }
    pub fn breakpoint_pc(&self) -> &str {
        &self.breakpoint_pc
    }
    pub fn flag_breakpoint_i(&self) -> bool {
        self.flag_breakpoint_i
    }
    pub fn breakpoint_i(&self) -> &str {
        &self.breakpoint_i
    }
    pub fn flag_breakpoint_opcode(&self) -> bool {
        self.flag_breakpoint_opcode
    }
    pub fn breakpoint_opcode(&self) -> &str {
        &self.breakpoint_opcode
    }

    pub fn handle_event<T>(&mut self, display: &Display, event: &Event<T>) {
        let gl_window = display.gl_window();
        self.platform
            .handle_event(self.imgui.io_mut(), gl_window.window(), &event);
    }

    pub fn prepare_frame(&mut self, display: &Display) -> Result<(), String> {
        self.platform.prepare_frame(self.imgui.io_mut(), display.gl_window().window())
            .map_err(|e| format!("Failed to prepare UI frame: {}", e))?;
        Ok(())
    }

    pub fn render<S: Surface>(
        &mut self,
        delta_time: Duration,
        display: &Display,
        target: &mut S,
        fps: f64,
        cpu: &CPU,
    ) -> Result<(), String> {
        self.is_open = false;
        self.imgui.io_mut().update_delta_time(delta_time);

        let mut reset_debug_layout = false;

        let about_name = &self.about_name;
        let about_version = &self.about_version;
        let about_description = &self.about_description;
        let about_license = &self.about_license;

        let window_width = display.gl_window().window().inner_size().width as f32;
        let window_height = display.gl_window().window().inner_size().height as f32;

        let ui = self.imgui.frame();
        let custom_font = ui.push_font(self.custom_font);
        if let Some(menu_bar) = ui.begin_main_menu_bar() {
            if let Some(menu) = ui.begin_menu(im_str!("File"), true) {
                self.is_open = true;
                MenuItem::new(im_str!("Open ROM or State..."))
                    .shortcut(im_str!("Ctrl + O"))
                    .build_with_ref(&ui, &mut self.flag_open);

                #[cfg(feature = "rom-download")]
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
                if let Some(menu) = ui.begin_menu(im_str!("Colors"), true) {
                    if ColorEdit::new(
                        im_str!("Background Color"),
                        self.color_settings.get_mut(Color::Background),
                    )
                    .build(&ui)
                    {
                        self.color_settings.changed = true;
                    }
                    if ColorEdit::new(
                        im_str!("Foreground Color"),
                        self.color_settings.get_mut(Color::Plane1),
                    )
                    .build(&ui)
                    {
                        self.color_settings.changed = true;
                    }
                    if ColorEdit::new(
                        im_str!("Foreground Color 2 (XO-CHIP)"),
                        self.color_settings.get_mut(Color::Plane2),
                    )
                    .build(&ui)
                    {
                        self.color_settings.changed = true;
                    }
                    if ColorEdit::new(
                        im_str!("Foreground Color 3 (XO-CHIP)"),
                        self.color_settings.get_mut(Color::PlaneBoth),
                    )
                    .build(&ui)
                    {
                        self.color_settings.changed = true;
                    }

                    ui.separator();

                    let mut preset_handler = ColorPresetHandler::new(&mut self.color_settings);
                    let mut color_changed = false;
                    if Self::menu_item_color_preset(
                        &ui,
                        &mut preset_handler,
                        im_str!("pich8 Default Preset"),
                        ColorPreset::Default,
                    ) {
                        color_changed = true;
                    }
                    if Self::menu_item_color_preset(
                        &ui,
                        &mut preset_handler,
                        im_str!("Octo Classic Preset"),
                        ColorPreset::OctoClassic,
                    ) {
                        color_changed = true;
                    }
                    if Self::menu_item_color_preset(
                        &ui,
                        &mut preset_handler,
                        im_str!("Octo LCD Preset"),
                        ColorPreset::OctoLcd,
                    ) {
                        color_changed = true;
                    }
                    if Self::menu_item_color_preset(
                        &ui,
                        &mut preset_handler,
                        im_str!("Octo Hotdog Preset"),
                        ColorPreset::OctoHotdog,
                    ) {
                        color_changed = true;
                    }
                    if Self::menu_item_color_preset(
                        &ui,
                        &mut preset_handler,
                        im_str!("Octo Gray Preset"),
                        ColorPreset::OctoGray,
                    ) {
                        color_changed = true;
                    }
                    if Self::menu_item_color_preset(
                        &ui,
                        &mut preset_handler,
                        im_str!("Octo CGA0 Preset"),
                        ColorPreset::OctoCga0,
                    ) {
                        color_changed = true;
                    }
                    if Self::menu_item_color_preset(
                        &ui,
                        &mut preset_handler,
                        im_str!("Octo CGA1 Preset"),
                        ColorPreset::OctoCga1,
                    ) {
                        color_changed = true;
                    }
                    if color_changed {
                        self.color_settings.changed = true;
                    }

                    menu.end(&ui);
                }
                ui.separator();
                MenuItem::new(im_str!("Display FPS"))
                    .shortcut(im_str!("F1"))
                    .build_with_ref(&ui, &mut self.flag_display_fps);
                MenuItem::new(im_str!("Debug"))
                    .shortcut(im_str!("F7"))
                    .build_with_ref(&ui, &mut self.flag_debug);
                if self.flag_debug {
                    MenuItem::new(im_str!("Reset Debug Window Layout"))
                        .build_with_ref(&ui, &mut reset_debug_layout);
                }
                menu.end(&ui);
            }
            if let Some(menu) = ui.begin_menu(im_str!("Settings"), true) {
                self.is_open = true;
                MenuItem::new(im_str!("Pause"))
                    .shortcut(im_str!("P"))
                    .build_with_ref(&ui, &mut self.flag_pause);
                ui.separator();
                if let Some(cpu_speed_menu) = ui.begin_menu(im_str!("CPU Speed"), true) {
                    Self::cpu_speed_menu_item(
                        &ui,
                        "Slowest",
                        420 * self.cpu_multiplier,
                        &mut self.cpu_speed,
                    );
                    Self::cpu_speed_menu_item(
                        &ui,
                        "Slow",
                        600 * self.cpu_multiplier,
                        &mut self.cpu_speed,
                    );
                    Self::cpu_speed_menu_item(
                        &ui,
                        "Normal",
                        720 * self.cpu_multiplier,
                        &mut self.cpu_speed,
                    );
                    Self::cpu_speed_menu_item(
                        &ui,
                        "Fast",
                        900 * self.cpu_multiplier,
                        &mut self.cpu_speed,
                    );
                    Self::cpu_speed_menu_item(
                        &ui,
                        "Faster",
                        1200 * self.cpu_multiplier,
                        &mut self.cpu_speed,
                    );
                    Self::cpu_speed_menu_item(
                        &ui,
                        "Fastest",
                        1500 * self.cpu_multiplier,
                        &mut self.cpu_speed,
                    );
                    ui.separator();
                    let before = self.cpu_multiplier == 50;
                    let mut after = before;
                    MenuItem::new(&im_str!("50x")).build_with_ref(&ui, &mut after);
                    if !before && after {
                        self.cpu_multiplier = 50;
                        self.cpu_speed *= 50;
                    } else if before && !after {
                        self.cpu_multiplier = 1;
                        self.cpu_speed /= 50;
                    }
                    cpu_speed_menu.end(&ui);
                }
                if let Some(quirks_menu) = ui.begin_menu(im_str!("Quirks"), true) {
                    MenuItem::new(im_str!("Load/Store"))
                        .build_with_ref(&ui, &mut self.quirks_settings.get_mut(Quirk::LoadStore));
                    MenuItem::new(im_str!("Shift"))
                        .build_with_ref(&ui, &mut self.quirks_settings.get_mut(Quirk::Shift));
                    MenuItem::new(im_str!("Draw"))
                        .build_with_ref(&ui, &mut self.quirks_settings.get_mut(Quirk::Draw));
                    MenuItem::new(im_str!("Jump0"))
                        .build_with_ref(&ui, &mut self.quirks_settings.get_mut(Quirk::Jump));
                    MenuItem::new(im_str!("VF Order"))
                        .build_with_ref(&ui, &mut self.quirks_settings.get_mut(Quirk::VfOrder));
                    MenuItem::new(im_str!("Partial Wrapping - Horizontal")).build_with_ref(
                        &ui,
                        &mut self.quirks_settings.get_mut(Quirk::PartialWrapH),
                    );
                    MenuItem::new(im_str!("Partial Wrapping - Vertical")).build_with_ref(
                        &ui,
                        &mut self.quirks_settings.get_mut(Quirk::PartialWrapV),
                    );
                    ui.separator();

                    let mut preset_handler = QuirksPresetHandler::new(&mut self.quirks_settings);
                    Self::menu_item_quirks_preset(
                        &ui,
                        &mut preset_handler,
                        im_str!("Default Preset (Legacy ROMs)"),
                        QuirksPreset::Default,
                    );
                    Self::menu_item_quirks_preset(
                        &ui,
                        &mut preset_handler,
                        im_str!("Octo Preset"),
                        QuirksPreset::Octo,
                    );

                    quirks_menu.end(&ui);
                }
                ui.separator();

                let mut vol = (self.volume * 100.0) as u8;
                Slider::new(im_str!("Audio Volume"))
                    .range(0..=100)
                    .display_format(im_str!("%d %%"))
                    .build(&ui, &mut vol);
                self.volume = vol as f32 / 100.0;

                MenuItem::new(im_str!("Mute Audio"))
                    .shortcut(im_str!("M"))
                    .build_with_ref(&ui, &mut self.flag_mute);
                menu.end(&ui);
            }
            if let Some(menu) = ui.begin_menu(im_str!("Help"), true) {
                self.is_open = true;
                MenuItem::new(im_str!("About")).build_with_ref(&ui, &mut self.flag_about);
                menu.end(&ui);
            }

            if self.flag_display_fps {
                let fps = im_str!("{:.0} fps", fps);
                let text_width = ui.calc_text_size(&fps, false, 0.0);
                ui.same_line(window_width - (text_width[0] * 1.25));
                ui.text_colored([0.75, 0.75, 0.75, 1.0], fps);
            }
            if self.flag_downloading {
                self.is_open = true;
                let text = im_str!("Downloading...");
                let text_size = ui.calc_text_size(text, false, 250.0);
                let dl_win_size = [text_size[0] + 50.0, text_size[1] + 40.0];
                let dl_win_pos = [
                    window_width / 2.0 - dl_win_size[0] / 2.0,
                    window_height / 2.0 - dl_win_size[1] / 2.0,
                ];
                Window::new(im_str!("Downloading"))
                    .position(dl_win_pos, Condition::Always)
                    .size(dl_win_size, Condition::Always)
                    .resizable(false)
                    .collapsible(false)
                    .movable(false)
                    .title_bar(false)
                    .build(&ui, || {
                        ui.set_cursor_pos([
                            dl_win_size[0] / 2.0 - text_size[0] / 2.0,
                            dl_win_size[1] / 2.0 - text_size[1] / 2.0,
                        ]);
                        ui.text_wrapped(&text);
                    });
            }
            if self.flag_about {
                self.is_open = true;
                let app_name_size = ui.calc_text_size(about_name, false, 0.0);
                let app_version_size = ui.calc_text_size(about_version, false, 0.0);
                let app_license_size = ui.calc_text_size(about_license, false, 0.0);
                let about_text_size = ui.calc_text_size(about_description, false, 250.0);
                let about_win_size = [
                    about_text_size[0] + 50.0,
                    about_text_size[1]
                        + app_name_size[1]
                        + app_version_size[1]
                        + app_license_size[1]
                        + 65.0,
                ];
                let about_win_pos = [
                    window_width / 2.0 - about_win_size[0] / 2.0,
                    window_height / 2.0 - about_win_size[1] / 2.0,
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
                        Self::centered_text(&ui, about_name, about_win_size[0]);
                        cfont_big.pop(&ui);

                        Self::centered_text(&ui, about_version, about_win_size[0]);

                        ui.spacing();
                        ui.set_cursor_pos([
                            about_win_size[0] / 2.0 - about_text_size[0] / 2.0,
                            ui.cursor_pos()[1],
                        ]);
                        ui.text_wrapped(about_description);

                        ui.spacing();
                        Self::centered_text(&ui, about_license, about_win_size[0]);
                    });
            }
            if self.flag_error {
                self.is_open = true;
                let text_size = ui.calc_text_size(&self.error_text, false, 250.0);
                let error_win_size = [text_size[0] + 50.0, text_size[1] + 40.0];
                let error_win_pos = [
                    window_width / 2.0 - error_win_size[0] / 2.0,
                    window_height / 2.0 - error_win_size[1] / 2.0,
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
                        ui.set_cursor_pos([
                            error_win_size[0] / 2.0 - text_size[0] / 2.0,
                            ui.cursor_pos()[1],
                        ]);
                        ui.text_wrapped(&error_text);
                    });
            }

            if self.flag_debug {
                let font = self.custom_font_small;
                let font = ui.push_font(font);

                let pos_condition = if reset_debug_layout {
                    Condition::Always
                } else {
                    Condition::FirstUseEver
                };

                let size = [130.0, 265.0];
                let pos = [10.0, 40.0];
                Window::new(im_str!("Registers"))
                    .position(pos, pos_condition)
                    .size(size, Condition::Always)
                    .resizable(false)
                    .build(&ui, || {
                        ui.columns(2, im_str!("registers"), true);

                        Self::register_col_u16(&ui, "PC", cpu.PC());
                        Self::register_col_u16(&ui, "I ", cpu.I());
                        Self::register_col_u8_greyed(&ui, "DT", cpu.DT(), cpu.DT() == 0);
                        Self::register_col_u8_greyed(&ui, "ST", cpu.ST(), cpu.ST() == 0);
                        ui.separator();
                        let v = cpu.V();
                        Self::register_col_u8(&ui, "V0", v[0]);
                        Self::register_col_u8(&ui, "V8", v[8]);
                        Self::register_col_u8(&ui, "V1", v[1]);
                        Self::register_col_u8(&ui, "V9", v[9]);
                        Self::register_col_u8(&ui, "V2", v[2]);
                        Self::register_col_u8(&ui, "VA", v[10]);
                        Self::register_col_u8(&ui, "V3", v[3]);
                        Self::register_col_u8(&ui, "VB", v[11]);
                        Self::register_col_u8(&ui, "V4", v[4]);
                        Self::register_col_u8(&ui, "VC", v[12]);
                        Self::register_col_u8(&ui, "V5", v[5]);
                        Self::register_col_u8(&ui, "VD", v[13]);
                        Self::register_col_u8(&ui, "V6", v[6]);
                        Self::register_col_u8(&ui, "VE", v[14]);
                        Self::register_col_u8(&ui, "V7", v[7]);
                        Self::register_col_u8(&ui, "VF", v[15]);
                    });

                let size = [130.0, 245.0];
                let pos = [window_width - size[0] - 10.0, 40.0];
                Window::new(im_str!("Stack"))
                    .position(pos, pos_condition)
                    .size(size, Condition::Always)
                    .resizable(false)
                    .build(&ui, || {
                        ui.columns(2, im_str!("stack"), true);
                        Self::register_col_u8(&ui, "SP", cpu.sp() as u8);
                        ui.next_column();
                        ui.separator();
                        let stack = cpu.stack();
                        Self::register_col_u16_greyed(&ui, "0 ", stack[0], cpu.sp() == 0);
                        Self::register_col_u16_greyed(&ui, "8 ", stack[8], cpu.sp() <= 8);
                        Self::register_col_u16_greyed(&ui, "1 ", stack[1], cpu.sp() <= 1);
                        Self::register_col_u16_greyed(&ui, "9 ", stack[9], cpu.sp() <= 9);
                        Self::register_col_u16_greyed(&ui, "2 ", stack[2], cpu.sp() <= 2);
                        Self::register_col_u16_greyed(&ui, "10", stack[10], cpu.sp() <= 10);
                        Self::register_col_u16_greyed(&ui, "3 ", stack[3], cpu.sp() <= 3);
                        Self::register_col_u16_greyed(&ui, "11", stack[11], cpu.sp() <= 11);
                        Self::register_col_u16_greyed(&ui, "4 ", stack[4], cpu.sp() <= 4);
                        Self::register_col_u16_greyed(&ui, "12", stack[12], cpu.sp() <= 12);
                        Self::register_col_u16_greyed(&ui, "5 ", stack[5], cpu.sp() <= 5);
                        Self::register_col_u16_greyed(&ui, "13", stack[13], cpu.sp() <= 13);
                        Self::register_col_u16_greyed(&ui, "6 ", stack[6], cpu.sp() <= 6);
                        Self::register_col_u16_greyed(&ui, "14", stack[14], cpu.sp() <= 14);
                        Self::register_col_u16_greyed(&ui, "7 ", stack[7], cpu.sp() <= 7);
                        Self::register_col_u16_greyed(&ui, "15", stack[15], cpu.sp() <= 15);
                    });

                let size = [260.0, 80.0];
                let pos = [
                    window_width / 3.0 - size[0] / 2.0,
                    window_height - size[1] - 10.0,
                ];
                let flag_breakpoint_pc = &mut self.flag_breakpoint_pc;
                let breakpoint_pc_im = &mut self.breakpoint_pc_im;
                let breakpoint_pc = &mut self.breakpoint_pc;
                let flag_breakpoint_i = &mut self.flag_breakpoint_i;
                let breakpoint_i_im = &mut self.breakpoint_i_im;
                let breakpoint_i = &mut self.breakpoint_i;
                let flag_breakpoint_opcode = &mut self.flag_breakpoint_opcode;
                let breakpoint_opcode_im = &mut self.breakpoint_opcode_im;
                let breakpoint_opcode = &mut self.breakpoint_opcode;
                Window::new(im_str!("Breakpoints"))
                    .position(pos, pos_condition)
                    .size(size, Condition::Always)
                    .resizable(false)
                    .build(&ui, || {
                        // Break on PC value
                        if Self::breakpoint_input(
                            &ui,
                            im_str!("PC"),
                            flag_breakpoint_pc,
                            breakpoint_pc_im,
                            true,
                        ) {
                            *breakpoint_pc = String::from(breakpoint_pc_im.to_str());
                        }

                        ui.same_line(0.0);
                        ui.dummy([30.0, 0.0]);

                        // Break on I value
                        ui.same_line(0.0);
                        if Self::breakpoint_input(
                            &ui,
                            im_str!("I "),
                            flag_breakpoint_i,
                            breakpoint_i_im,
                            true,
                        ) {
                            *breakpoint_i = String::from(breakpoint_i_im.to_str());
                        }

                        // Break on opcode
                        if Self::breakpoint_input(
                            &ui,
                            im_str!("Opcode"),
                            flag_breakpoint_opcode,
                            breakpoint_opcode_im,
                            false,
                        ) {
                            // Sanitize and fill input
                            let mut value: String = breakpoint_opcode_im
                                .to_str()
                                .chars()
                                .map(|c| match c {
                                    '0'..='9' => c,
                                    'A'..='F' => c,
                                    _ => '*',
                                })
                                .collect();
                            while value.len() < 4 {
                                value.insert(0, '*');
                            }

                            let mut new_im = ImString::with_capacity(4);
                            new_im.push_str(&value);
                            *breakpoint_opcode_im = new_im;
                            *breakpoint_opcode = value;
                        }
                    });

                let size = [260.0, 80.0];
                let pos = [
                    2.0 * window_width / 3.0 - size[0] / 2.0,
                    window_height - size[1] - 10.0,
                ];
                Window::new(im_str!("Opcodes"))
                    .position(pos, pos_condition)
                    .size(size, Condition::Always)
                    .resizable(false)
                    .build(&ui, || {
                        Self::opcode_text(
                            &ui,
                            "> Next",
                            cpu.next_opcode(),
                            &cpu.next_opcode_description(),
                        );
                        let style =
                            ui.push_style_color(StyleColor::Text, Self::COLOR_TEXT_DISABLED);
                        Self::opcode_text(&ui, "  Last", cpu.opcode(), &cpu.opcode_description());
                        style.pop(&ui);
                    });

                let size = [347.0, 37.0];
                let pos = [
                    window_width / 2.0 - size[0] / 2.0,
                    self.last_menu_height as f32 + 10.0,
                ];
                let mut pause = &mut self.flag_pause;
                let step = &mut self.flag_step;
                let step_timers = &mut self.flag_step_timers;
                Window::new(im_str!("Debug"))
                    .position(pos, Condition::Always)
                    .size(size, Condition::Always)
                    .resizable(false)
                    .title_bar(false)
                    .build(&ui, || {
                        let button_size = [105.0, 20.0];
                        Self::toggle_button(&ui, im_str!("Pause (P)"), button_size, &mut pause);
                        ui.same_line(0.0);
                        if Self::button_disabled(&ui, im_str!("Step (F8)"), button_size, !*pause) {
                            *step = true;
                        }
                        ui.same_line(0.0);
                        if Self::button_disabled(
                            &ui,
                            im_str!("Step Timers (F9)"),
                            button_size,
                            !*pause,
                        ) {
                            *step_timers = true;
                        }
                    });

                font.pop(&ui);
            }

            // Store menu bar height with a bit of clearance
            self.last_menu_height = ui.window_size()[1] as u32 + Self::MENU_HEIGHT_CLEARANCE;

            custom_font.pop(&ui);
            menu_bar.end(&ui);
        }

        let gl_window = display.gl_window();
        self.platform.prepare_render(&ui, gl_window.window());

        let draw_data = ui.render();
        self.renderer
            .render(target, draw_data)
            .map_err(|e| format!("Failed to render UI: {}", e))?;

        Ok(())
    }

    fn register_col_u16(ui: &Ui, name: &str, value: u16) {
        ui.align_text_to_frame_padding();
        ui.text(name);
        ui.same_line(0.0);
        let mut inp = ImString::new(format!("{:04X}", value));
        let width = ui.push_item_width(Self::WIDTH_TEXTBOX_REGISTER);
        ui.input_text(&ImString::from(format!("##{}", name)), &mut inp)
            .read_only(true)
            .build();
        width.pop(&ui);
        ui.next_column();
    }

    fn register_col_u8(ui: &Ui, name: &str, value: u8) {
        ui.align_text_to_frame_padding();
        ui.text(name);
        ui.same_line(0.0);
        let mut inp = ImString::new(format!("{:02X}", value));
        let width = ui.push_item_width(Self::WIDTH_TEXTBOX_REGISTER);
        ui.input_text(&ImString::from(format!("##{}", name)), &mut inp)
            .read_only(true)
            .build();
        width.pop(&ui);
        ui.next_column();
    }

    fn register_col_u16_greyed(ui: &Ui, name: &str, value: u16, greyed: bool) {
        let mut style = None;
        if greyed {
            style = Some(ui.push_style_color(StyleColor::Text, Self::COLOR_TEXT_DISABLED));
        }
        ui.align_text_to_frame_padding();
        ui.text(name);
        ui.same_line(0.0);
        let mut inp = ImString::new(format!("{:04X}", value));
        let width = ui.push_item_width(Self::WIDTH_TEXTBOX_REGISTER);
        ui.input_text(&ImString::from(format!("##{}", name)), &mut inp)
            .read_only(true)
            .build();
        width.pop(&ui);
        ui.next_column();
        if let Some(style) = style {
            style.pop(&ui);
        }
    }

    fn register_col_u8_greyed(ui: &Ui, name: &str, value: u8, greyed: bool) {
        let mut style = None;
        if greyed {
            style = Some(ui.push_style_color(StyleColor::Text, Self::COLOR_TEXT_DISABLED));
        }
        ui.align_text_to_frame_padding();
        ui.text(name);
        ui.same_line(0.0);
        let mut inp = ImString::new(format!("{:02X}", value));
        let width = ui.push_item_width(Self::WIDTH_TEXTBOX_REGISTER);
        ui.input_text(&ImString::from(format!("##{}", name)), &mut inp)
            .read_only(true)
            .build();
        width.pop(&ui);
        ui.next_column();
        if let Some(style) = style {
            style.pop(&ui);
        }
    }

    fn opcode_text(ui: &Ui, name: &str, value: u16, description: &str) {
        ui.align_text_to_frame_padding();
        ui.text(name);
        ui.same_line(0.0);
        let mut inp = ImString::new(format!("{:04X}", value));
        let width = ui.push_item_width(Self::WIDTH_TEXTBOX_REGISTER);
        ui.input_text(&ImString::from(format!("##{}", name)), &mut inp)
            .read_only(true)
            .build();
        width.pop(&ui);
        ui.same_line(0.0);
        ui.text(description);
    }

    fn breakpoint_input(
        ui: &Ui,
        name: &ImStr,
        enabled: &mut bool,
        value: &mut ImString,
        hex_chars: bool,
    ) -> bool {
        ui.checkbox(name, enabled);
        ui.same_line(0.0);
        let width = ui.push_item_width(Self::WIDTH_TEXTBOX_REGISTER);
        ui.input_text(&ImString::from(format!("##{}", name)), value)
            .chars_hexadecimal(hex_chars)
            .chars_uppercase(true)
            .resize_buffer(false)
            .build();
        width.pop(&ui);
        ui.is_item_deactivated_after_edit()
    }

    fn toggle_button(ui: &Ui, text: &ImStr, size: [f32; 2], active: &mut bool) {
        if *active {
            let col0 = ui.push_style_color(
                StyleColor::Button,
                [15.0 / 255.0, 135.0 / 255.0, 250.0 / 255.0, 1.0],
            );
            let col1 = ui.push_style_color(
                StyleColor::ButtonHovered,
                [66.0 / 255.0, 150.0 / 255.0, 250.0 / 255.0, 1.0],
            );
            let col2 = ui.push_style_color(
                StyleColor::ButtonActive,
                [41.0 / 255.0, 74.0 / 255.0, 122.0 / 255.0, 0.75],
            );
            if ui.button(text, size) {
                *active = !*active;
            }
            col0.pop(&ui);
            col1.pop(&ui);
            col2.pop(&ui);
        } else {
            let col0 = ui.push_style_color(
                StyleColor::Button,
                [41.0 / 255.0, 74.0 / 255.0, 122.0 / 255.0, 0.75],
            );
            let col1 = ui.push_style_color(
                StyleColor::ButtonHovered,
                [66.0 / 255.0, 150.0 / 255.0, 250.0 / 255.0, 0.75],
            );
            let col2 = ui.push_style_color(
                StyleColor::ButtonActive,
                [15.0 / 255.0, 135.0 / 255.0, 250.0 / 255.0, 1.0],
            );
            if ui.button(text, size) {
                *active = !*active;
            }
            col0.pop(&ui);
            col1.pop(&ui);
            col2.pop(&ui);
        }
    }

    fn button_disabled(ui: &Ui, text: &ImStr, size: [f32; 2], disabled: bool) -> bool {
        if disabled {
            ui.same_line(0.0);
            let col0 = ui.push_style_color(
                imgui::StyleColor::Button,
                [41.0 / 255.0, 74.0 / 255.0, 122.0 / 255.0, 0.25],
            );
            let col1 = ui.push_style_color(
                imgui::StyleColor::ButtonHovered,
                [41.0 / 255.0, 74.0 / 255.0, 122.0 / 255.0, 0.25],
            );
            let col2 = ui.push_style_color(
                imgui::StyleColor::ButtonActive,
                [41.0 / 255.0, 74.0 / 255.0, 122.0 / 255.0, 0.25],
            );
            let res = ui.button(text, size);
            col0.pop(&ui);
            col1.pop(&ui);
            col2.pop(&ui);
            res
        } else {
            ui.button(text, size)
        }
    }

    fn centered_text(ui: &Ui, text: &ImStr, window_width: f32) {
        let text_width = ui.calc_text_size(text, false, 0.0)[0];
        ui.set_cursor_pos([window_width / 2.0 - text_width / 2.0, ui.cursor_pos()[1]]);
        ui.text_wrapped(&text);
    }

    pub fn menu_height(&self) -> u32 {
        self.last_menu_height
    }

    fn cpu_speed_menu_item(ui: &Ui, name: &str, item_speed: u32, current_speed: &mut u32) {
        let mut flag = *current_speed == item_speed;
        MenuItem::new(&im_str!("{} ({}Hz)", name, item_speed)).build_with_ref(ui, &mut flag);
        if flag {
            *current_speed = item_speed;
        }
    }

    fn menu_item_color_preset(
        ui: &Ui,
        preset_handler: &mut ColorPresetHandler,
        name: &ImStr,
        preset: ColorPreset,
    ) -> bool {
        let active = &mut preset_handler.is_active(preset);
        MenuItem::new(name).build_with_ref(&ui, active);
        if *active {
            preset_handler.set_preset(preset);
            true
        } else {
            false
        }
    }

    fn menu_item_quirks_preset(
        ui: &Ui,
        preset_handler: &mut QuirksPresetHandler,
        name: &ImStr,
        preset: QuirksPreset,
    ) {
        let active = &mut preset_handler.is_active(preset);
        MenuItem::new(name).build_with_ref(&ui, active);
        if *active {
            preset_handler.set_preset(preset);
        }
    }

    pub fn display_error(&mut self, message: &str) {
        self.flag_error = true;
        self.error_text = ImString::new(message);
    }
}
