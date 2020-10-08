use crate::cpu::CPU;
use crate::display::WindowDisplay;
use crate::contracts::{DisplayOutput, SoundOutput};
use crate::sound::{NoSound, BeepSound};
use std::thread::sleep;
use std::time::{Duration, Instant};
use minifb::Key;

pub struct Emulator<T: SoundOutput> {
    cpu: CPU,
    display: WindowDisplay,
    sound: T,
    last_tick: Instant,
}

impl Emulator<BeepSound> {
    pub fn new() -> Self {
        Self{
            cpu: CPU::new(),
            display: WindowDisplay::new(),
            sound: BeepSound::new(440),
            last_tick: Instant::now(),
        }
    }
}

impl Emulator<NoSound> {
    pub fn new_without_sound() -> Self {
        Self{
            cpu: CPU::new(),
            display: WindowDisplay::new(),
            sound: NoSound{},
            last_tick: Instant::now(),
        }
    }
}

impl<T: SoundOutput> Emulator<T> {
    const FRAMES_PER_SEC: f64 = 60.0;
    const TICKS_PER_SEC: u16 = 720;

    pub fn run(&mut self, rom: &[u8]) {
        self.cpu.load_rom(rom);
        self.run_loop();
    }

    pub fn run_state(&mut self, state: &[u8]) {
        self.cpu = CPU::from_state(state).unwrap(); //TODO: Error handling
        self.run_loop();
    }

    fn run_loop(&mut self) {
        while self.display.is_open() {
            self.last_tick = Instant::now();

            for _ in 0..(Emulator::<T>::TICKS_PER_SEC / Emulator::<T>::FRAMES_PER_SEC as u16) {
                self.cpu.tick(&self.get_input());
            }
            self.cpu.update_timers();
            
            if self.cpu.draw() {
                self.display.draw(self.cpu.vmem());
                self.cpu.set_draw(false);
            } else {
                self.display.update();
            }
            if self.cpu.sound_active() {
                //TODO: beep
            }

            self.sleep();
        }
    }

    fn sleep(&mut self) {
        let elapsed = self.last_tick.elapsed().as_nanos() as f64;
        let sleep_time = (1_000_000_000.0 / Emulator::<T>::FRAMES_PER_SEC) - elapsed;
        if sleep_time > 0.0 {
            sleep(Duration::from_nanos(sleep_time as u64));
        }
    }

    fn get_input(&self) -> [bool; 16] {
        let keys = &[
            Key::Key1,  Key::Key2,  Key::Key3,  Key::Key4,
            Key::Q,     Key::W,     Key::E,     Key::R,
            Key::A,     Key::S,     Key::D,     Key::F,
            Key::Z,     Key::X,     Key::C,     Key::V,
        ];
        let mut input = [false; 16];
        for (key, i) in keys.iter().zip(0..16) {
            input[i] = self.display.is_key_down(key.to_owned());
        }
        input
    }
}