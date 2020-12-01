use crate::cpu::{Breakpoint, CPU};
use crate::dialog_handler::{DialogHandler, FileDialogResult, FileDialogType};
use crate::display::WindowDisplay;
use crate::fps_counter::FpsCounter;
use crate::gui::GUI;
use crate::gui::{Color, Quirk};
use crate::sound::AudioPlayer;
use glium::glutin::{
    event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use std::{fs, time::Instant};

#[cfg(feature = "rom-download")]
use crate::rom_downloader::{DownloadResult, RomDownloader};

enum LoadedType {
    Nothing,
    Rom(Vec<u8>),
    State(Vec<u8>),
}

pub struct Emulator {
    cpu: CPU,
    cpu_speed: u32,
    display: WindowDisplay,
    gui: GUI,
    sound: AudioPlayer,
    fps_counter: FpsCounter,
    mute: bool,
    input: [bool; 16],
    loaded: LoadedType,
    pause: bool,
    step: bool,
    step_timers: bool,
    frame_time: Instant,
    last_timer: Instant,
    last_cycle: Instant,
    pause_time: Instant,
    dialog_handler: DialogHandler,
    modifiers_state: ModifiersState,
    last_correction_cpu: Instant,
    counter_cpu: u32,
    last_correction_timer: Instant,
    counter_timer: u32,
    force_redraw: bool,

    #[cfg(feature = "rom-download")]
    rom_downloader: RomDownloader,
}

impl Emulator {
    const CPU_FREQUENCY: u16 = 720;
    const TIMER_FREQUENCY: u8 = 60;
    const NANOS_PER_TIMER: u64 = 1_000_000_000 / Emulator::TIMER_FREQUENCY as u64;
    const MAX_FILE_SIZE: u32 = u16::MAX as u32 + 10000;

    pub fn new(event_loop: &EventLoop<()>, vsync: bool) -> Result<Self, String> {
        let display = WindowDisplay::new(&event_loop, vsync)?;
        let mut cpu = CPU::new();
        cpu.load_bootrom();
        cpu.draw = true;
        let cpu_speed = Emulator::CPU_FREQUENCY as u32;

        // Initialize GUI
        let mut gui = GUI::new(display.display());
        gui.cpu_speed = cpu_speed;
        gui.volume = 0.25;

        let now = Instant::now();
        Ok(Self {
            cpu,
            cpu_speed,
            display,
            gui,
            sound: AudioPlayer::new().expect("Failed to create sound output device"),
            mute: false,
            input: [false; 16],
            loaded: LoadedType::Nothing,
            pause: false,
            step: false,
            step_timers: false,
            frame_time: now,
            last_timer: now,
            last_cycle: now,
            pause_time: now,
            dialog_handler: DialogHandler::new(),
            fps_counter: FpsCounter::new(),
            modifiers_state: ModifiersState::empty(),
            last_correction_cpu: Instant::now(),
            counter_cpu: 0,
            last_correction_timer: Instant::now(),
            counter_timer: 0,
            force_redraw: true,

            #[cfg(feature = "rom-download")]
            rom_downloader: RomDownloader::new(),
        })
    }

    fn reset(&mut self) {
        match &self.loaded {
            LoadedType::Rom(rom) => {
                self.cpu = CPU::new();
                match self.cpu.load_rom(&rom) {
                    Ok(_) => {
                        if !self.gui.flag_debug {
                            self.gui.flag_pause = false;
                        }
                    }
                    Err(_) => self.gui.display_error("Data is not a valid ROM!"),
                }
            }
            LoadedType::State(state) => {
                match CPU::from_state(&state) {
                    Ok(cpu) => self.cpu = cpu,
                    Err(msg) => self.gui.display_error(&msg),
                }
                self.gui.flag_pause = false;
            }
            _ => (),
        }
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        self.loaded = LoadedType::Rom(rom.to_vec());
        self.reset();
    }

    pub fn load_state(&mut self, state: &[u8]) {
        self.loaded = LoadedType::State(state.to_vec());
        self.reset();
    }

    fn set_pause(&mut self, pause: bool) {
        self.pause = pause;
        if pause {
            // Store timestamp
            self.pause_time = Instant::now();
        } else {
            // "Subtract" paused time so the simulation doesn't jump
            let diff = Instant::now() - self.pause_time;
            self.last_cycle += diff;
        }
    }

    #[cfg(feature = "rom-download")]
    fn handle_downloads(&mut self) {
        if self.rom_downloader.is_active() {
            match self.rom_downloader.check_result() {
                DownloadResult::Success(data) => {
                    self.gui.flag_downloading = false;
                    self.load_rom(&data);
                }
                DownloadResult::Fail(msg) => {
                    self.gui.flag_downloading = false;
                    self.gui.display_error(&msg);
                }
                DownloadResult::None => (),
            }
        }
    }

    pub fn handle_event(&mut self, event: Event<()>, ctrl_flow: &mut ControlFlow) {
        // Handle file dialogs
        if self.dialog_handler.is_open() {
            match self.dialog_handler.check_result() {
                FileDialogResult::OpenRom(file_path) => {
                    match fs::metadata(&file_path) {
                        Ok(metadata) => {
                            if metadata.len() <= Self::MAX_FILE_SIZE as u64 {
                                match fs::read(&file_path) {
                                    Ok(file) => {
                                        // Check if it's a p8s state file, otherwise expect ROM
                                        if &file[0..3] == b"p8s" {
                                            self.load_state(&file[3..]);
                                        } else {
                                            self.load_rom(&file);
                                        }
                                    }
                                    Err(err) => self.gui.display_error(&format!("Error: {}", err)),
                                }
                            } else {
                                self.gui.display_error("File is too big!");
                            }
                        }
                        Err(err) => self.gui.display_error(&format!("Error: {}", err)),
                    }
                }
                FileDialogResult::SaveState(file_path) => match self.cpu.save_state().as_mut() {
                    Ok(state) => {
                        state.splice(0..0, b"p8s".iter().cloned());
                        if fs::write(file_path, state).is_err() {
                            self.gui.display_error("Failed to write to file!");
                        }
                    }
                    Err(msg) => self.gui.display_error(&msg),
                },

                #[cfg(feature = "rom-download")]
                FileDialogResult::InputUrl(url) => {
                    self.gui.flag_downloading = true;
                    match url::Url::parse(&url) {
                        Ok(url) => self.rom_downloader.download(url),
                        Err(e) => self.gui.display_error(&format!("Invalid URL: {}", e)),
                    }
                }

                FileDialogResult::None => (),
            }
        }

        // Handle downloads
        #[cfg(feature = "rom-download")]
        self.handle_downloads();

        // Handle events
        if !self.dialog_handler.is_open() {
            self.gui.handle_event(self.display.display(), &event);
            match event {
                Event::NewEvents(_) => {
                    self.handle_gui_flags(ctrl_flow);
                }
                Event::MainEventsCleared => {
                    if !self.pause {
                        // Perform emulation
                        let nanos_per_cycle = 1_000_000_000 / self.cpu_speed as u64;
                        if self.last_cycle.elapsed().as_nanos() as u64 >= nanos_per_cycle * 10 {
                            let mut cycles = (self.last_cycle.elapsed().as_nanos() as f64
                                / nanos_per_cycle as f64)
                                as u32;
                            self.last_cycle = Instant::now();

                            // Check if additional cycles are needed
                            if self.last_correction_cpu.elapsed().as_secs_f64() >= 0.25 {
                                let target = self.cpu_speed / 4;
                                if self.counter_cpu < target {
                                    cycles += target - self.counter_cpu;
                                }
                                self.last_correction_cpu = Instant::now();
                                self.counter_cpu = 0;
                            } else {
                                self.counter_cpu += cycles;
                            }

                            for _ in 0..cycles {
                                if let Err(e) = self.cpu.tick(&self.input) {
                                    self.gui.display_error(&format!("Error: {}", e));
                                    continue;
                                }
                                if self.gui.flag_debug && self.check_breakpoints() {
                                    self.gui.flag_pause = true;
                                    break;
                                }
                            }
                        }
                        // Update CPU timers
                        if self.last_timer.elapsed().as_nanos() as u64 >= Emulator::NANOS_PER_TIMER
                        {
                            self.last_timer = Instant::now();
                            let mut reps = 1;

                            // Check and correct frequency regularly
                            if self.last_correction_timer.elapsed().as_secs_f64() >= 0.25 {
                                let target = Self::TIMER_FREQUENCY as u32 / 4;
                                if self.counter_timer + 1 < target {
                                    reps += target - self.counter_timer - 1;
                                }
                                self.last_correction_timer = Instant::now();
                                self.counter_timer = 0;
                            } else {
                                self.counter_timer += reps;
                            }

                            for _ in 0..reps {
                                if self.cpu.ST() > 0 && !self.mute {
                                    if self.cpu.audio_buffer().is_some() {
                                        self.sound.play_buffer(self.cpu.audio_buffer().unwrap());
                                    } else {
                                        self.sound.beep();
                                    }
                                }
                                self.cpu.update_timers();
                            }
                        }
                    } else if self.step {
                        if let Err(e) = self.cpu.tick(&self.input) {
                            self.gui.display_error(&format!("Error: {}", e));
                        }
                    } else if self.step_timers {
                        self.cpu.update_timers();
                    }

                    // Always request redrawing to keep the GUI updated
                    self.display.display().gl_window().window().request_redraw();
                }
                Event::RedrawRequested(_) => {
                    let fps = self.fps_counter.tick();
                    let frame_duration = Instant::now() - self.frame_time;
                    self.frame_time = Instant::now();

                    let is_fullscreen = self.display.fullscreen();
                    let height = if is_fullscreen {
                        0
                    } else {
                        self.gui.menu_height()
                    };
                    let vmem = if self.force_redraw || self.cpu.draw {
                        self.cpu.draw = false;
                        Some(self.cpu.vmem())
                    } else {
                        None
                    };
                    let mut frame = self
                        .display
                        .prepare(vmem, height)
                        .expect("Failed to prepare frame");
                    if !is_fullscreen {
                        self.gui
                            .render(
                                frame_duration,
                                self.display.display(),
                                &mut frame,
                                fps,
                                &self.cpu,
                            )
                            .expect("Failed to render GUI");
                    }
                    self.display.render(frame).expect("Failed to render frame");
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { input, .. },
                    ..
                } => self.handle_input(input, ctrl_flow),
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *ctrl_flow = ControlFlow::Exit,
                Event::WindowEvent {
                    event: WindowEvent::ModifiersChanged(modifiers_state),
                    ..
                } => self.modifiers_state = modifiers_state,
                _ => (),
            }
        }
    }

    #[inline]
    fn handle_gui_flags(&mut self, ctrl_flow: &mut ControlFlow) {
        let fullscreen = self.display.fullscreen();
        let mut pause = false;

        if self.gui.is_open() && !fullscreen {
            // Pause emulation while gui menu/window is open
            pause = true;
        }

        if self.gui.flag_open {
            self.dialog_handler
                .open_file_dialog(FileDialogType::OpenRom);
            self.gui.flag_open = false;
        }

        #[cfg(feature = "rom-download")]
        if self.gui.flag_open_rom_url {
            self.dialog_handler
                .open_file_dialog(FileDialogType::InputUrl);
            self.gui.flag_open_rom_url = false;
        }

        if self.gui.flag_save_state {
            self.dialog_handler
                .open_file_dialog(FileDialogType::SaveState);
            self.gui.flag_save_state = false;
        }
        if self.gui.flag_reset {
            self.reset();
            self.gui.flag_reset = false;
        }
        if self.gui.flag_exit {
            *ctrl_flow = ControlFlow::Exit;
            self.gui.flag_exit = false;
        }
        if self.gui.flag_fullscreen != fullscreen {
            let _ = self.display.toggle_fullscreen();
        }
        if self.gui.flag_pause {
            pause = true;
        }

        let color_settings = self.gui.color_settings();
        self.force_redraw = color_settings.changed;
        if color_settings.changed {
            color_settings.changed = false;

            let color_bg = color_settings.get(Color::Background);
            self.display.color_bg = [
                (color_bg[0] * 255.0) as u8,
                (color_bg[1] * 255.0) as u8,
                (color_bg[2] * 255.0) as u8,
            ];
            let color_plane_1 = color_settings.get(Color::Plane1);
            self.display.color_plane_1 = [
                (color_plane_1[0] * 255.0) as u8,
                (color_plane_1[1] * 255.0) as u8,
                (color_plane_1[2] * 255.0) as u8,
            ];
            let color_plane_2 = color_settings.get(Color::Plane2);
            self.display.color_plane_2 = [
                (color_plane_2[0] * 255.0) as u8,
                (color_plane_2[1] * 255.0) as u8,
                (color_plane_2[2] * 255.0) as u8,
            ];
            let color_plane_both = color_settings.get(Color::PlaneBoth);
            self.display.color_plane_both = [
                (color_plane_both[0] * 255.0) as u8,
                (color_plane_both[1] * 255.0) as u8,
                (color_plane_both[2] * 255.0) as u8,
            ];
        }

        self.cpu_speed = self.gui.cpu_speed as u32;
        self.cpu.vertical_wrapping = self.gui.flag_vertical_wrapping;
        self.mute = self.gui.flag_mute;
        self.sound.set_volume(self.gui.volume);

        let quirks = self.gui.quirks_settings();
        self.cpu.quirk_load_store = quirks.get(Quirk::LoadStore);
        self.cpu.quirk_shift = quirks.get(Quirk::Shift);
        self.cpu.quirk_draw = quirks.get(Quirk::Draw);
        self.cpu.quirk_jump = quirks.get(Quirk::Jump);
        self.cpu.quirk_vf_order = quirks.get(Quirk::VfOrder);

        self.step = self.gui.flag_step;
        self.gui.flag_step = false;
        self.step_timers = self.gui.flag_step_timers;
        self.gui.flag_step_timers = false;

        if pause != self.pause {
            self.set_pause(pause);
        }
    }

    #[inline]
    fn check_breakpoints(&mut self) -> bool {
        // Check breakpoints
        use std::u16;
        if self.gui.flag_breakpoint_pc() {
            if let Ok(bp) = u16::from_str_radix(self.gui.breakpoint_pc(), 16) {
                if self.cpu.check_breakpoint(Breakpoint::PC(bp)) {
                    return true;
                }
            }
        }
        if self.gui.flag_breakpoint_i() {
            if let Ok(bp) = u16::from_str_radix(self.gui.breakpoint_i(), 16) {
                if self.cpu.check_breakpoint(Breakpoint::I(bp)) {
                    return true;
                }
            }
        }
        if self.gui.flag_breakpoint_opcode()
            && self
                .cpu
                .check_breakpoint(Breakpoint::Opcode(self.gui.breakpoint_opcode().to_string()))
        {
            return true;
        }
        false
    }

    #[inline]
    fn handle_input(
        &mut self,
        KeyboardInput {
            scancode,
            virtual_keycode,
            state,
            ..
        }: KeyboardInput,
        ctrl_flow: &mut ControlFlow,
    ) {
        use ElementState::*;
        use VirtualKeyCode::*;
        const SCANCODE_1: u32 = 2;
        const SCANCODE_2: u32 = 3;
        const SCANCODE_3: u32 = 4;
        const SCANCODE_4: u32 = 5;
        const SCANCODE_Q: u32 = 16;
        const SCANCODE_W: u32 = 17;
        const SCANCODE_E: u32 = 18;
        const SCANCODE_R: u32 = 19;
        const SCANCODE_A: u32 = 30;
        const SCANCODE_S: u32 = 31;
        const SCANCODE_D: u32 = 32;
        const SCANCODE_F: u32 = 33;
        const SCANCODE_Z: u32 = 44;
        const SCANCODE_X: u32 = 45;
        const SCANCODE_C: u32 = 46;
        const SCANCODE_V: u32 = 47;

        if let Some(keycode) = virtual_keycode {
            let ctrl = self.modifiers_state.ctrl();
            let shift = self.modifiers_state.shift();
            match (scancode, keycode, state, ctrl, shift) {
                // Command keys
                #[cfg(feature = "rom-download")]
                (_, O, Pressed, true, true) => {
                    self.gui.flag_open_rom_url = true;
                }

                (_, Escape, Pressed, _, _) => {
                    if self.gui.flag_fullscreen {
                        self.gui.flag_fullscreen = false;
                    } else {
                        *ctrl_flow = ControlFlow::Exit;
                    }
                }
                (_, F1, Pressed, _, _) => {
                    self.gui.flag_display_fps = !self.gui.flag_display_fps;
                }
                (_, F5, Pressed, _, _) => {
                    self.gui.flag_reset = true;
                }
                (_, F7, Pressed, _, _) => {
                    self.gui.flag_debug = !self.gui.flag_debug;
                }
                (_, F8, Pressed, _, _) => {
                    self.gui.flag_step = true;
                }
                (_, F9, Pressed, _, _) => {
                    self.gui.flag_step_timers = true;
                }
                (_, F11, Pressed, _, _) => {
                    self.gui.flag_fullscreen = !self.gui.flag_fullscreen;
                }
                (_, P, Pressed, _, _) => {
                    self.gui.flag_pause = !self.gui.flag_pause;
                }
                (_, M, Pressed, _, _) => {
                    self.gui.flag_mute = !self.gui.flag_mute;
                }
                (_, O, Pressed, true, _) => {
                    self.gui.flag_open = true;
                }
                (_, S, Pressed, true, _) => {
                    self.gui.flag_save_state = true;
                }

                // Chip8 keys - using scancode instead of VirtualKeyCode to account for different keyboard layouts
                (SCANCODE_1, _, Pressed, _, _) => self.input[1] = true,
                (SCANCODE_1, _, Released, _, _) => self.input[1] = false,
                (SCANCODE_2, _, Pressed, _, _) => self.input[2] = true,
                (SCANCODE_2, _, Released, _, _) => self.input[2] = false,
                (SCANCODE_3, _, Pressed, _, _) => self.input[3] = true,
                (SCANCODE_3, _, Released, _, _) => self.input[3] = false,
                (SCANCODE_4, _, Pressed, _, _) => self.input[0xC] = true,
                (SCANCODE_4, _, Released, _, _) => self.input[0xC] = false,
                (SCANCODE_Q, _, Pressed, _, _) => self.input[4] = true,
                (SCANCODE_Q, _, Released, _, _) => self.input[4] = false,
                (SCANCODE_W, _, Pressed, _, _) => self.input[5] = true,
                (SCANCODE_W, _, Released, _, _) => self.input[5] = false,
                (SCANCODE_E, _, Pressed, _, _) => self.input[6] = true,
                (SCANCODE_E, _, Released, _, _) => self.input[6] = false,
                (SCANCODE_R, _, Pressed, _, _) => self.input[0xD] = true,
                (SCANCODE_R, _, Released, _, _) => self.input[0xD] = false,
                (SCANCODE_A, _, Pressed, _, _) => self.input[7] = true,
                (SCANCODE_A, _, Released, _, _) => self.input[7] = false,
                (SCANCODE_S, _, Pressed, _, _) => self.input[8] = true,
                (SCANCODE_S, _, Released, _, _) => self.input[8] = false,
                (SCANCODE_D, _, Pressed, _, _) => self.input[9] = true,
                (SCANCODE_D, _, Released, _, _) => self.input[9] = false,
                (SCANCODE_F, _, Pressed, _, _) => self.input[0xE] = true,
                (SCANCODE_F, _, Released, _, _) => self.input[0xE] = false,
                (SCANCODE_Z, _, Pressed, _, _) => self.input[0xA] = true,
                (SCANCODE_Z, _, Released, _, _) => self.input[0xA] = false,
                (SCANCODE_X, _, Pressed, _, _) => self.input[0] = true,
                (SCANCODE_X, _, Released, _, _) => self.input[0] = false,
                (SCANCODE_C, _, Pressed, _, _) => self.input[0xB] = true,
                (SCANCODE_C, _, Released, _, _) => self.input[0xB] = false,
                (SCANCODE_V, _, Pressed, _, _) => self.input[0xF] = true,
                (SCANCODE_V, _, Released, _, _) => self.input[0xF] = false,

                _ => (),
            }
        }
    }
}
