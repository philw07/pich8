use crate::cpu::CPU;
use crate::display::WindowDisplay;
use crate::gui::GUI;
use crate::sound::BeepSound;
use crate::file_dialog_handler::{FileDialogHandler, FileDialogResult, FileDialogType};
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
    cpu_speed: u16,
    display: WindowDisplay,
    gui: GUI,
    sound: BeepSound,
    mute: bool,
    input: BitArray<Msb0, [u16; 1]>,
    loaded: LoadedType,
    pause: bool,
    frame_time: Instant,
    last_timer: Instant,
    last_cycle: Instant,
    pause_time: Instant,
    file_dialog: FileDialogHandler,
}

impl Emulator {
    const CPU_FREQUENCY: u16 = 720;
    const TIMER_FREQUENCY: u8 = 60;
    const NANOS_PER_TIMER: u64 = 1_000_000_000 / Emulator::TIMER_FREQUENCY as u64;

    pub fn new(event_loop: &EventLoop<()>) -> Result<Self, String> {
        let display = WindowDisplay::new(&event_loop, false)?;
        let cpu = CPU::new();
        let cpu_speed = Emulator::CPU_FREQUENCY;

        // Initialize GUI
        let mut gui = GUI::new(display.display());
        gui.set_flag_quirk_load_store(cpu.quirk_load_store());
        gui.set_flag_quirk_shift(cpu.quirk_shift());
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
            file_dialog: FileDialogHandler::new(),
        })
    }

    fn reset(&mut self) {
        match &self.loaded {
            LoadedType::Rom(rom) => {
                self.cpu = CPU::new();
                self.cpu.load_rom(&rom);
                self.gui.set_flag_pause(false);
            },
            LoadedType::State(state) => {
                self.cpu = CPU::from_state(&state).expect("Failed to load state");
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
        if self.file_dialog.is_open() && self.file_dialog.check_result() {
            match self.file_dialog.last_result() {
                FileDialogResult::OpenRom(file_path) => {
                    if let Ok(file) = fs::read(&file_path) {
                        self.load_rom(&file);
                    }
                },
                FileDialogResult::LoadState(file_path) => {
                    if let Ok(file) = fs::read(&file_path) {
                        self.load_state(&file);
                    }
                },
                FileDialogResult::SaveState(file_path) => {
                    let state = self.cpu.save_state().expect("Failed to save state");
                    fs::write(file_path, state).expect("Failed to write file");
                },
                FileDialogResult::None => (),
            }
        }

        // Handle events
        if !self.file_dialog.is_open() {
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
                                self.cpu.tick(&self.input);
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
                    let frame_duration = Instant::now() - self.frame_time;
                    self.frame_time = Instant::now();

                    let is_fullscreen = self.display.fullscreen();
                    let height = if is_fullscreen { 0 } else { self.gui.menu_height() };
                    let mut frame = self.display.prepare(self.cpu.vmem(), height).expect("Failed to render frame");
                    if !is_fullscreen {
                        self.gui.render(frame_duration, self.display.display(), &mut frame).expect("Failed to render GUI");
                    }
                    self.display.render(frame).expect("Faield to render frame");
                },
                Event::WindowEvent{ event: WindowEvent::KeyboardInput { input: KeyboardInput{ scancode, virtual_keycode: Some(keycode), state, .. }, .. }, .. } => self.handle_input(scancode, keycode, state, ctrl_flow),
                Event::WindowEvent{ event: WindowEvent::CloseRequested, .. } => *ctrl_flow = ControlFlow::Exit,
                _ => (),
            }
        }
    }

    #[inline]
    fn handle_gui_flags(&mut self, ctrl_flow: &mut ControlFlow) {
        let fullscreen = self.display.fullscreen();
        let mut pause = false;

        if self.gui.menu_open() && !fullscreen {
            // Pause emulation while menu is open
            pause = true;
        }

        if self.gui.flag_open_rom() {
            self.file_dialog.open_dialog(FileDialogType::OpenRom);
            self.gui.set_flag_open_rom(false);
        }
        if self.gui.flag_load_state() {
            self.file_dialog.open_dialog(FileDialogType::LoadState);
            self.gui.set_flag_load_state(false);
        }
        if self.gui.flag_save_state() {
            self.file_dialog.open_dialog(FileDialogType::SaveState);
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

        self.cpu_speed = self.gui.cpu_speed();
        self.cpu.set_quirk_load_store(self.gui.flag_quirk_load_store());
        self.cpu.set_quirk_shift(self.gui.flag_quirk_shift());
        self.cpu.set_vertical_wrapping(self.gui.flag_vertical_wrapping());
        self.mute = self.gui.flag_mute();

        if pause != self.pause {
            self.set_pause(pause);
        }
    }

    #[inline]
    fn handle_input(&mut self, scancode: u32, keycode: VirtualKeyCode, state: ElementState, ctrl_flow: &mut ControlFlow) {
        use VirtualKeyCode::*;
        use ElementState::*;
        match (scancode, keycode, state) {
            // Command keys
            (_, Escape, Pressed) => {
                if self.gui.flag_fullscreen() {
                    self.gui.set_flag_fullscreen(false);
                } else {
                    *ctrl_flow = ControlFlow::Exit;
                }
            },
            (_, F11, Pressed) => { self.gui.set_flag_fullscreen(!self.gui.flag_fullscreen()); },
            (_, P, Pressed) => { self.gui.set_flag_pause(!self.gui.flag_pause()); },
            (_, M, Pressed) => { self.gui.set_flag_mute(!self.gui.flag_mute()); },

            // Chip8 keys - using scancode instead of VirtualKeyCode to account for different keyboard layouts
            (2, _, Pressed)     => self.input.set(0, true),
            (2, _, Released)    => self.input.set(0, false),
            (3, _, Pressed)     => self.input.set(1, true),
            (3, _, Released)    => self.input.set(1, false),
            (4, _, Pressed)     => self.input.set(2, true),
            (4, _, Released)    => self.input.set(2, false),
            (5, _, Pressed)     => self.input.set(3, true),
            (5, _, Released)    => self.input.set(3, false),
            (16, _, Pressed)    => self.input.set(4, true),
            (16, _, Released)   => self.input.set(4, false),
            (17, _, Pressed)    => self.input.set(5, true),
            (17, _, Released)   => self.input.set(5, false),
            (18, _, Pressed)    => self.input.set(6, true),
            (18, _, Released)   => self.input.set(6, false),
            (19, _, Pressed)    => self.input.set(7, true),
            (19, _, Released)   => self.input.set(7, false),
            (30, _, Pressed)    => self.input.set(8, true),
            (30, _, Released)   => self.input.set(8, false),
            (31, _, Pressed)    => self.input.set(9, true),
            (31, _, Released)   => self.input.set(9, false),
            (32, _, Pressed)    => self.input.set(10, true),
            (32, _, Released)   => self.input.set(10, false),
            (33, _, Pressed)    => self.input.set(11, true),
            (33, _, Released)   => self.input.set(11, false),
            (44, _, Pressed)    => self.input.set(12, true),
            (44, _, Released)   => self.input.set(12, false),
            (45, _, Pressed)    => self.input.set(13, true),
            (45, _, Released)   => self.input.set(13, false),
            (46, _, Pressed)    => self.input.set(14, true),
            (46, _, Released)   => self.input.set(14, false),
            (47, _, Pressed)    => self.input.set(15, true),
            (47, _, Released)   => self.input.set(15, false),

            _ => (),
        }
    }
}