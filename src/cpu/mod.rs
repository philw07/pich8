use std::fmt;
use rand::prelude::*;
use bitvec::prelude::*;
use getset::{CopyGetters, Getters, Setters};
use serde::{Serialize, Deserialize};
use crate::serde_big_array::BigArray;
use crate::video_memory::{VideoMemory, VideoMode};

mod opcodes;

#[derive(Debug)]
pub enum Error {
    SaveStateError(rmp_serde::encode::Error),
    LoadStateError(rmp_serde::decode::Error),
    ProgramCounterOverflow,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Error::SaveStateError(e) => write!(f, "Save state error: {}", e),
            Error::LoadStateError(e) => write!(f, "Load state error: {}", e),
            Error::ProgramCounterOverflow => write!(f, "Program counter overflow"),
        }
    }
}

impl From<rmp_serde::encode::Error> for Error {
    fn from(err: rmp_serde::encode::Error) -> Error {
        Error::SaveStateError(err)
    }
}
impl From<rmp_serde::decode::Error> for Error {
    fn from(err: rmp_serde::decode::Error) -> Error {
        Error::LoadStateError(err)
    }
}

#[allow(non_snake_case)]
#[derive(CopyGetters, Getters, Setters, Serialize, Deserialize)]
pub struct CPU {
    #[serde(with = "BigArray")]
    mem: [u8; 4096],                        // Main memory
    #[getset(get = "pub")]
    vmem: VideoMemory,                      // Graphics memory
    stack: [u16; 16],                       // Stack to store locations before a jump occurs
    keys: BitArray<Msb0, [u16; 1]>,         // Keypad status

    PC: u16,                                // Program counter
    V: [u8; 16],                            // Registers
    I: u16,                                 // Index register
    DT: u8,                                 // Delay timer
    ST: u8,                                 // Sound timer
    RPL: [u8; 8],                           // HP48 RPL flags (used for S-CHIP)

    opcode: u16,                            // Current opcode
    sp: usize,                              // Current stack position

    #[getset(get_copy = "pub", set = "pub")]
    draw: bool,                             // Drawing flag
    key_wait: bool,                         // Key wait flag
    key_reg: usize,                         // Key wait register
    #[getset(get_copy = "pub", set = "pub")]
    quirk_load_store: bool,                 // Flag for load store quirk
    #[getset(get_copy = "pub", set = "pub")]
    quirk_shift: bool,                      // Flag for shift quirk
    #[getset(get_copy = "pub", set = "pub")]
    quirk_jump: bool,                       // Flag for jump0 quirk
    #[getset(get_copy = "pub", set = "pub")]
    quirk_vf_order: bool,                   // Flag for VF order quirk
    // Originally, a 16x16 sprite is only drawn if n == 0 AND extended display mode (128x64) is active (CHIP8.DOC by David Winter).
    // In default mode (64x32) however, if n (height) == 0, a 8x16 pixels sprite is drawn.
    // However, Octo and many other emulators only check for n == 0, so some ROMs (e.g. Eaty the Alien) assume this check instead.
    #[getset(get_copy = "pub", set = "pub")]
    quirk_draw: bool,                       // Flag for draw quirk
    #[getset(get_copy = "pub", set = "pub")]
    vertical_wrapping: bool,                // Flag for vertical wrapping
}

impl CPU {
    const BOOTROM: &'static [u8] = include_bytes!("../../data/bootrom/pich8-logo.ch8");
    const PC_INITIAL: u16 = 0x200;
    const FONTSET: &'static [u8] = &[ 
        0xF0, 0x90, 0x90, 0x90, 0xF0,   // 0
        0x20, 0x60, 0x20, 0x20, 0x70,   // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0,   // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0,   // 3
        0x90, 0x90, 0xF0, 0x10, 0x10,   // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0,   // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0,   // 6
        0xF0, 0x10, 0x20, 0x40, 0x40,   // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0,   // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0,   // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90,   // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0,   // B
        0xF0, 0x80, 0x80, 0x80, 0xF0,   // C
        0xE0, 0x90, 0x90, 0x90, 0xE0,   // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0,   // E
        0xF0, 0x80, 0xF0, 0x80, 0x80    // F
    ];
    const FONTSET_SUPER: &'static [u8] = &[
        0x3C, 0x7E, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0x7E, 0x3C, // 0
        0x18, 0x38, 0x58, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, // 1
        0x3E, 0x7F, 0xC3, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xFF, 0xFF, // 2
        0x3C, 0x7E, 0xC3, 0x03, 0x0E, 0x0E, 0x03, 0xC3, 0x7E, 0x3C, // 3
        0x06, 0x0E, 0x1E, 0x36, 0x66, 0xC6, 0xFF, 0xFF, 0x06, 0x06, // 4
        0xFF, 0xFF, 0xC0, 0xC0, 0xFC, 0xFE, 0x03, 0xC3, 0x7E, 0x3C, // 5
        0x3E, 0x7C, 0xC0, 0xC0, 0xFC, 0xFE, 0xC3, 0xC3, 0x7E, 0x3C, // 6
        0xFF, 0xFF, 0x03, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x60, 0x60, // 7
        0x3C, 0x7E, 0xC3, 0xC3, 0x7E, 0x7E, 0xC3, 0xC3, 0x7E, 0x3C, // 8
        0x3C, 0x7E, 0xC3, 0xC3, 0x7F, 0x3F, 0x03, 0x03, 0x3E, 0x7C, // 9
    ];

    pub fn new() -> Self
    {
        let mut cpu = Self {
            mem: [0; 4096],
            vmem: VideoMemory::new(),
            stack: [0; 16],
            keys: bitarr![Msb0, u16; 0; 16],

            PC: CPU::PC_INITIAL,
            V: [0; 16],
            I: 0,
            DT: 0,
            ST: 0,
            RPL: [0; 8],

            opcode: 0,
            sp: 0,

            draw: false,
            key_wait: false,
            key_reg: 0,
            quirk_load_store: true,
            quirk_shift: true,
            quirk_draw: true,
            quirk_jump: true,
            quirk_vf_order: true,
            vertical_wrapping: true,
        };
        
        // Load fontsets
        &cpu.mem[0..Self::FONTSET.len()].copy_from_slice(Self::FONTSET);
        &cpu.mem[0x50..0x50+Self::FONTSET_SUPER.len()].copy_from_slice(Self::FONTSET_SUPER);
        
        cpu
    }

    pub fn from_state(state: &[u8]) -> Result<Self, String> {
        Ok(rmp_serde::decode::from_slice(state).map_err(|_| "Failed to deserialize state!")?)
    }

    pub fn save_state(&self) -> Result<Vec<u8>, String> {
        Ok(rmp_serde::encode::to_vec(self).map_err(|_| "Failed to serialize state!")?)
    }

    pub fn load_bootrom(&mut self) {
        self.load_rom(Self::BOOTROM).unwrap();
    }

    pub fn load_rom(&mut self, prog: &[u8]) -> Result<(), String> {
        if prog.len() <= self.mem.len() - 0x200 {
            self.vmem.set_video_mode(VideoMode::Default);
            &self.mem[0x200..0x200+prog.len()].copy_from_slice(prog);
            self.PC = CPU::PC_INITIAL;
            Ok(())
        } else {
            self.load_bootrom();
            Err("Invalid ROM!".to_string())
        }
    }

    pub fn sound_active(&self) -> bool {
        self.ST > 0
    }

    pub fn update_timers(&mut self) {
        if self.DT > 0 {
            self.DT -= 1;
        }
        if self.ST > 0 {
            self.ST -= 1;
        }
    }

    pub fn tick(&mut self, keys: &BitArray<Msb0, [u16; 1]>) -> Result<(), Error> {
        self.keys.copy_from_bitslice(keys);
        if self.key_wait {
            for i in 0..keys.len() {
                if keys[i] {
                    self.key_wait = false;
                    self.V[self.key_reg] = i as u8;
                }
            }
        }

        if !self.key_wait {
            self.emulate_cycle()
        } else {
            Ok(())
        }
    }

    fn emulate_cycle(&mut self) -> Result<(), Error> {
        // Fetch opcode
        if self.PC as usize >= self.mem.len() - 1 {
            self.load_bootrom();
            return Err(Error::ProgramCounterOverflow);
        }
        self.opcode = (self.mem[self.PC as usize] as u16) << 8 | (self.mem[(self.PC + 1) as usize] as u16);

        // Decode opcode
        let x = (self.opcode & 0x0F00) as usize >> 8;
        let y = (self.opcode & 0x00F0) as usize >> 4;
        let n = (self.opcode & 0x000F) as u8;
        let nn = (self.opcode & 0x00FF) as u8;
        let nnn = (self.opcode & 0x0FFF) as u16;
        let last_nibble = self.opcode & 0x000F;

        // Execute opcode
        match self.opcode & 0xF000 {
            0x0000 => match nnn {
                0x0C1..=0x0CF => self.opcode_schip_0x00CN(n),
                0x0E0 => self.opcode_0x00E0(),
                0x0EE => self.opcode_0x00EE(),
                0x0FB => self.opcode_schip_0x00FB(),
                0x0FC => self.opcode_schip_0x00FC(),
                0x0FD => self.opcode_schip_0x00FD(),
                0x0FE => self.opcode_schip_0x00FE(),
                0x0FF => self.opcode_schip_0x00FF(),
                0x230 => self.opcode_hires_0x0230(),
                _ => self.opcode_0x0NNN(),
            },
            0x1000 => match nnn {
                0x260 => self.opcode_0x1260(nnn),
                _ => self.opcode_0x1NNN(nnn),
            },
            0x2000 => self.opcode_0x2NNN(nnn),
            0x3000 => self.opcode_0x3XNN(x, nn),
            0x4000 => self.opcode_0x4XNN(x, nn),
            0x5000 => match last_nibble {
                0x0 => self.opcode_0x5XY0(x, y),
                _ => self.opcode_invalid(),
            },
            0x6000 => self.opcode_0x6XNN(x, nn),
            0x7000 => self.opcode_0x7XNN(x, nn),
            0x8000 => match self.opcode & 0x000F {
                0x0 => self.opcode_0x8XY0(x, y),
                0x1 => self.opcode_0x8XY1(x, y),
                0x2 => self.opcode_0x8XY2(x, y),
                0x3 => self.opcode_0x8XY3(x, y),
                0x4 => self.opcode_0x8XY4(x, y),
                0x5 => self.opcode_0x8XY5(x, y),
                0x6 => self.opcode_0x8XY6(x, y),
                0x7 => self.opcode_0x8XY7(x, y),
                0xE => self.opcode_0x8XYE(x, y),
                _ => self.opcode_invalid(),
            },
            0x9000 => match last_nibble {
                0x0 => self.opcode_0x9XY0(x, y),
                _ => self.opcode_invalid(),
            }
            0xA000 => self.opcode_0xANNN(nnn),
            0xB000 => self.opcode_0xBNNN(nnn),
            0xC000 => self.opcode_0xCXNN(x, nn),
            0xD000 => self.opcode_0xDXYN(x, y, last_nibble as usize),
            0xE000 => match nn {
                0x9E => self.opcode_0xEX9E(x),
                0xA1 => self.opcode_0xEXA1(x),
                _ => self.opcode_invalid(),
            },
            0xF000 => match nn {
                    0x07 => self.opcode_0xFX07(x),
                    0x0A => self.opcode_0xFX0A(x),
                    0x15 => self.opcode_0xFX15(x),
                    0x18 => self.opcode_0xFX18(x),
                    0x1E => self.opcode_0xFX1E(x),
                    0x29 => self.opcode_0xFX29(x),
                    0x30 => self.opcode_schip_0xFX30(x),
                    0x33 => self.opcode_0xFX33(x),
                    0x55 => self.opcode_0xFX55(x),
                    0x65 => self.opcode_0xFX65(x),
                    0x75 => self.opcode_schip_0xFX75(x),
                    0x85 => self.opcode_schip_0xFX85(x),
                    _ => self.opcode_invalid(),
            },
            _ => self.opcode_invalid(),
        };
        Ok(())
    }

    fn draw_sprite(&mut self, x: usize, y: usize, height: usize) {
        let big_sprite = (self.vmem.video_mode() == &VideoMode::Extended || self.quirk_draw) && height == 0;
        let step = if big_sprite { 2 } else { 1 };
        let width = if big_sprite { 16 } else { 8 };
        let height = if height == 0 { 16 } else { height };

        let sprite = &self.mem[self.I as usize..self.I as usize + (width/8) * height];
        let mut collision = false;

        for (k, mut y) in (0..sprite.len()).step_by(step).zip(y..y+height) {
            // Wrap around
            let last_line = self.vmem.height();
            if y >= last_line {
                if self.vertical_wrapping {
                    y %= last_line;
                } else {
                    continue;
                }
            }

            for (mut x, i) in (x..x+width).zip((0..width).rev()) {
                // Wrap around
                x %= self.vmem.width();

                // Get bit
                let bit = if width == 16 {
                    ((sprite[k] as u16) << 8 | sprite[k+1] as u16) >> i & 0b1 > 0
                } else {
                    sprite[k] >> i & 0b1 > 0
                };

                // Detect collision and draw pixel
                if bit && self.vmem.get(x, y) {
                    collision = true;
                }
                let res = self.vmem.get(x, y) != bit;
                self.vmem.set(x, y, res);
            }
        }

        self.V[0xF] = collision as u8;
    }
}

#[cfg(test)]
#[path = "./test.rs"]
mod cpu_test;
