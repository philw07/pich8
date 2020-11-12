use crate::cpu::CPU;
use crate::display::WindowDisplay;
use crate::gui::GUI;
use crate::sound::BeepSound;
use crate::dialog_handler::{DialogHandler, FileDialogResult, FileDialogType};
use crate::fps_counter::FpsCounter;
use crate::rom_downloader::{RomDownloader, DownloadResult};
use std::{time::Instant, fs};
use bitvec::prelude::*;
use glium::{
    glutin::{
        event_loop::{
            EventLoop,
            ControlFlow,
        },
        event::{
            Event,
            WindowEvent,
            KeyboardInput,
            VirtualKeyCode,
            ElementState,
            ModifiersState,
        },
    },
};

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
    sound: BeepSound,
    fps_counter: FpsCounter,
    mute: bool,
    input: BitArray<Msb0, [u16; 1]>,
    loaded: LoadedType,
    pause: bool,
    frame_time: Instant,
    last_timer: Instant,
    last_cycle: Instant,
    pause_time: Instant,
    dialog_handler: DialogHandler,
    modifiers_state: ModifiersState,
    rom_downloader: RomDownloader,
}

impl Emulator {
    const CPU_FREQUENCY: u16 = 720;
    const TIMER_FREQUENCY: u8 = 60;
    const NANOS_PER_TIMER: u64 = 1_000_000_000 / Emulator::TIMER_FREQUENCY as u64;
    const MAX_FILE_SIZE: u16 = 8192;

    pub fn new(event_loop: &EventLoop<()>, vsync: bool) -> Result<Self, String> {
        let display = WindowDisplay::new(&event_loop, vsync)?;
        let mut cpu = CPU::new();
        cpu.load_bootrom();
        let cpu_speed = Emulator::CPU_FREQUENCY as u32;

        // Initialize GUI
        let mut gui = GUI::new(display.display());
        gui.set_flag_quirk_load_store(cpu.quirk_load_store());
        gui.set_flag_quirk_shift(cpu.quirk_shift());
        gui.set_flag_quirk_draw(cpu.quirk_draw());
        gui.set_flag_quirk_jump(cpu.quirk_jump());
        gui.set_flag_quirk_vf_order(cpu.quirk_vf_order());
        gui.set_flag_vertical_wrapping(cpu.vertical_wrapping());
        gui.set_cpu_speed(cpu_speed);

        let bg_color = display.bg_color();
        gui.set_bg_color([
            bg_color[0] as f32 / 255.0,
            bg_color[1] as f32 / 255.0,
            bg_color[2] as f32 / 255.0
        ]);
        let fg_color = display.fg_color();
        gui.set_fg_color([
            fg_color[0] as f32 / 255.0,
            fg_color[1] as f32 / 255.0,
            fg_color[2] as f32 / 255.0
        ]);

        let now = Instant::now();
        Ok(Self{
            cpu,
            cpu_speed,
            display,
            gui,
            sound: BeepSound::new().expect("Failed to create sound output device"),
            mute: false,
            input: bitarr![Msb0, u16; 0; 16],
            loaded: LoadedType::Nothing,
            pause: false,
            frame_time: now,
            last_timer: now,
            last_cycle: now,
            pause_time: now,
            dialog_handler: DialogHandler::new(),
            fps_counter: FpsCounter::new(),
            modifiers_state: ModifiersState::empty(),
            rom_downloader: RomDownloader::new(),
        })
    }

    fn reset(&mut self) {
        match &self.loaded {
            LoadedType::Rom(rom) => {
                self.cpu = CPU::new();
                match self.cpu.load_rom(&rom) {
                    Ok(_) => { self.gui.set_flag_pause(false); },
                    Err(_) => self.gui.display_error("Data is not a valid ROM!"),
                }
            },
            LoadedType::State(state) => {
                match CPU::from_state(&state) {
                    Ok(cpu) => self.cpu = cpu,
                    Err(msg) => self.gui.display_error(&msg),
                }
                self.gui.set_flag_pause(false);
            },
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
                                        if &file[0..3] == "p8s".as_bytes() {
                                            self.load_state(&file[3..]);
                                        } else {
                                            self.load_rom(&file);
                                        }
                                    },
                                    Err(err) => self.gui.display_error(&format!("Error: {}", err)),
                                }
                            } else {
                                self.gui.display_error("File is too big!");
                            }
                        },
                        Err(err) => self.gui.display_error(&format!("Error: {}", err)),
                    }
                },
                FileDialogResult::InputUrl(url) => {
                    self.gui.set_flag_downloading(true);
                    match url::Url::parse(&url) {
                        Ok(url) => self.rom_downloader.download(url),
                        Err(e) => self.gui.display_error(&format!("Invalid URL: {}", e)),    
                    }
                }
                FileDialogResult::SaveState(file_path) => {
                    match self.cpu.save_state().as_mut() {
                        Ok(state) => {
                            state.splice(0..0, "p8s".as_bytes().iter().cloned());
                            if fs::write(file_path, state).is_err() {
                                self.gui.display_error("Failed to write to file!");
                            }
                        },
                        Err(msg) => self.gui.display_error(&msg),
                    }
                },
                FileDialogResult::None => (),
            }
        }

        // Handle downloads
        if self.rom_downloader.is_active() {
            match self.rom_downloader.check_result() {
                DownloadResult::Success(data) => {
                    self.gui.set_flag_downloading(false);
                    self.load_rom(&data);
                },
                DownloadResult::Fail(msg) => {
                    self.gui.set_flag_downloading(false);
                    self.gui.display_error(&msg);
                },
                DownloadResult::None => (),
            }
        }

        // Handle events
        if !self.dialog_handler.is_open() {
            self.gui.handle_event(self.display.display(), &event);
            match event {
                Event::NewEvents(_) => {
                    self.handle_gui_flags(ctrl_flow);
                },
                Event::MainEventsCleared => {
                    if !self.pause {
                        // Perform 
                        let nanos_per_cycle = 1_000_000_000 / self.cpu_speed as u64;
                        if self.last_cycle.elapsed().as_nanos() as u64 >= nanos_per_cycle * 10 {
                            let cycles = (self.last_cycle.elapsed().as_nanos() as f64 / nanos_per_cycle as f64) as u64;
                            self.last_cycle = Instant::now();
                            for _ in 0..cycles {
                                if let Err(e) = self.cpu.tick(&self.input) {
                                    self.gui.display_error(&format!("Error: {}", e));
                                }
                            }
                        }
                        // Update CPU timers
                        if self.last_timer.elapsed().as_nanos() as u64 >= Emulator::NANOS_PER_TIMER {
                            if self.cpu.sound_active() && !self.mute {
                                self.sound.beep();
                            }
                            self.last_timer = Instant::now();
                            self.cpu.update_timers();
                        }
                    }

                    // Always request redrawing to keep the GUI updated
                    self.display.display().gl_window().window().request_redraw();
                },
                Event::RedrawRequested(_) => {
                    let fps = self.fps_counter.tick();
                    let frame_duration = Instant::now() - self.frame_time;
                    self.frame_time = Instant::now();

                    let is_fullscreen = self.display.fullscreen();
                    let height = if is_fullscreen { 0 } else { self.gui.menu_height() };
                    let mut frame = self.display.prepare(self.cpu.vmem(), height).expect("Failed to prepare frame");
                    if !is_fullscreen {
                        self.gui.render(frame_duration, self.display.display(), &mut frame, fps).expect("Failed to render GUI");
                    }
                    self.display.render(frame).expect("Failed to render frame");
                },
                Event::WindowEvent{ event: WindowEvent::KeyboardInput { input, .. }, .. } => self.handle_input(input, ctrl_flow),
                Event::WindowEvent{ event: WindowEvent::CloseRequested, .. } => *ctrl_flow = ControlFlow::Exit,
                Event::WindowEvent{ event: WindowEvent::ModifiersChanged(modifiers_state), .. } => self.modifiers_state = modifiers_state,
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

        if self.gui.flag_open() {
            self.dialog_handler.open_file_dialog(FileDialogType::OpenRom);
            self.gui.set_flag_open(false);
        }
        if self.gui.flag_open_rom_url() {
            self.dialog_handler.open_file_dialog(FileDialogType::InputUrl);
            self.gui.set_flag_open_rom_url(false);
        }
        if self.gui.flag_save_state() {
            self.dialog_handler.open_file_dialog(FileDialogType::SaveState);
            self.gui.set_flag_save_state(false);
        }
        if self.gui.flag_reset() {
            self.reset();
            self.gui.set_flag_reset(false);
        }
        if self.gui.flag_exit() {
            *ctrl_flow = ControlFlow::Exit;
            self.gui.set_flag_exit(false);
        }
        if self.gui.flag_fullscreen() != fullscreen {
            let _ = self.display.toggle_fullscreen();
        }
        if self.gui.flag_pause() {
            pause = true;
        }

        let bg_color = self.gui.bg_color();
        self.display.set_bg_color([
            (bg_color[0] * 255.0) as u8,
            (bg_color[1] * 255.0) as u8,
            (bg_color[2] * 255.0) as u8
        ]);
        let fg_color = self.gui.fg_color();
        self.display.set_fg_color([
            (fg_color[0] * 255.0) as u8,
            (fg_color[1] * 255.0) as u8,
            (fg_color[2] * 255.0) as u8
        ]);

        self.cpu_speed = self.gui.cpu_speed() as u32;
        self.cpu.set_quirk_load_store(self.gui.flag_quirk_load_store());
        self.cpu.set_quirk_shift(self.gui.flag_quirk_shift());
        self.cpu.set_quirk_draw(self.gui.flag_quirk_draw());
        self.cpu.set_quirk_jump(self.gui.flag_quirk_jump());
        self.cpu.set_quirk_vf_order(self.gui.flag_quirk_vf_order());
        self.cpu.set_vertical_wrapping(self.gui.flag_vertical_wrapping());
        self.mute = self.gui.flag_mute();

        if pause != self.pause {
            self.set_pause(pause);
        }
    }

    #[inline]
    fn handle_input(&mut self, KeyboardInput { scancode, virtual_keycode, state, .. }: KeyboardInput, ctrl_flow: &mut ControlFlow) {
        use VirtualKeyCode::*;
        use ElementState::*;
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
                (_, Escape, Pressed, _, _) => {
                    if self.gui.flag_fullscreen() {
                        self.gui.set_flag_fullscreen(false);
                    } else {
                        *ctrl_flow = ControlFlow::Exit;
                    }
                },
                (_, F11, Pressed, _, _) => { self.gui.set_flag_fullscreen(!self.gui.flag_fullscreen()); },
                (_, F5, Pressed, _, _) => { self.gui.set_flag_reset(true); },
                (_, P, Pressed, _, _) => { self.gui.set_flag_pause(!self.gui.flag_pause()); },
                (_, M, Pressed, _, _) => { self.gui.set_flag_mute(!self.gui.flag_mute()); },
                (_, O, Pressed, true, true) => { self.gui.set_flag_open_rom_url(true); },
                (_, O, Pressed, true, _) => { self.gui.set_flag_open(true); },
                (_, S, Pressed, true, _) => { self.gui.set_flag_save_state(true); },

                // Chip8 keys - using scancode instead of VirtualKeyCode to account for different keyboard layouts
                (SCANCODE_1, _, Pressed, _, _)     => self.input.set(1, true),
                (SCANCODE_1, _, Released, _, _)    => self.input.set(1, false),
                (SCANCODE_2, _, Pressed, _, _)     => self.input.set(2, true),
                (SCANCODE_2, _, Released, _, _)    => self.input.set(2, false),
                (SCANCODE_3, _, Pressed, _, _)     => self.input.set(3, true),
                (SCANCODE_3, _, Released, _, _)    => self.input.set(3, false),
                (SCANCODE_4, _, Pressed, _, _)     => self.input.set(0xC, true),
                (SCANCODE_4, _, Released, _, _)    => self.input.set(0xC, false),
                (SCANCODE_Q, _, Pressed, _, _)    => self.input.set(4, true),
                (SCANCODE_Q, _, Released, _, _)   => self.input.set(4, false),
                (SCANCODE_W, _, Pressed, _, _)    => self.input.set(5, true),
                (SCANCODE_W, _, Released, _, _)   => self.input.set(5, false),
                (SCANCODE_E, _, Pressed, _, _)    => self.input.set(6, true),
                (SCANCODE_E, _, Released, _, _)   => self.input.set(6, false),
                (SCANCODE_R, _, Pressed, _, _)    => self.input.set(0xD, true),
                (SCANCODE_R, _, Released, _, _)   => self.input.set(0xD, false),
                (SCANCODE_A, _, Pressed, _, _)    => self.input.set(7, true),
                (SCANCODE_A, _, Released, _, _)   => self.input.set(7, false),
                (SCANCODE_S, _, Pressed, _, _)    => self.input.set(8, true),
                (SCANCODE_S, _, Released, _, _)   => self.input.set(8, false),
                (SCANCODE_D, _, Pressed, _, _)    => self.input.set(9, true),
                (SCANCODE_D, _, Released, _, _)   => self.input.set(9, false),
                (SCANCODE_F, _, Pressed, _, _)    => self.input.set(0xE, true),
                (SCANCODE_F, _, Released, _, _)   => self.input.set(0xE, false),
                (SCANCODE_Z, _, Pressed, _, _)    => self.input.set(0xA, true),
                (SCANCODE_Z, _, Released, _, _)   => self.input.set(0xA, false),
                (SCANCODE_X, _, Pressed, _, _)    => self.input.set(0, true),
                (SCANCODE_X, _, Released, _, _)   => self.input.set(0, false),
                (SCANCODE_C, _, Pressed, _, _)    => self.input.set(0xB, true),
                (SCANCODE_C, _, Released, _, _)   => self.input.set(0xB, false),
                (SCANCODE_V, _, Pressed, _, _)    => self.input.set(0xF, true),
                (SCANCODE_V, _, Released, _, _)   => self.input.set(0xF, false),

                _ => (),
            }
        }
    }
}