use std::fmt;
use rand::prelude::*;
use bitvec::prelude::*;
use getset::{CopyGetters, Getters, Setters};
use serde::{Serialize, Deserialize};
use crate::serde_big_array::BigArray;

mod opcodes;

#[derive(Debug)]
pub enum Error {
    SaveStateError(rmp_serde::encode::Error),
    LoadStateError(rmp_serde::decode::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Error::SaveStateError(e) => write!(f, "save state error: {}", e),
            Error::LoadStateError(e) => write!(f, "load state error: {}", e),
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
#[derive(CopyGetters, Getters, Setters, Serialize, Deserialize, Debug)]
pub struct CPU {
    #[serde(with = "BigArray")]
    mem: [u8; 4096],                    // Main memory
    #[getset(get = "pub")]
    vmem: BitArray<Msb0, [u64; 32]>,    // Graphics memory
    stack: [u16; 16],                   // Stack to store locations before a jump occurs
    keys: BitArray<Msb0, [u16; 1]>,     // Keypad status

    PC: u16,                            // Program counter
    V: [u8; 16],                        // Registers
    I: u16,                             // Index register
    DT: u8,                             // Delay timer
    ST: u8,                             // Sound timer

    opcode: u16,                        // Current opcode
    sp: usize,                          // Current stack position

    #[getset(get_copy = "pub", set = "pub")]
    draw: bool,                         // Drawing flag
    key_wait: bool,                     // Key wait flag
    key_reg: usize,                     // Key wait register
    #[getset(get_copy = "pub", set = "pub")]
    quirk_load_store: bool,             // Flag for load store quirk
    #[getset(get_copy = "pub", set = "pub")]
    quirk_shift: bool,                  // Flag for shift quirk
    #[getset(get_copy = "pub", set = "pub")]
    vertical_wrapping: bool,            // Flag for vertical wrapping
}

impl CPU {
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

    pub fn new() -> Self
    {
        let mut cpu = Self {
            mem: [0; 4096],
            vmem: bitarr![Msb0, u64; 0; 64*32],
            stack: [0; 16],
            keys: bitarr![Msb0, u16; 0; 16],

            PC: CPU::PC_INITIAL,
            V: [0; 16],
            I: 0,
            DT: 0,
            ST: 0,

            opcode: 0,
            sp: 0,

            draw: false,
            key_wait: false,
            key_reg: 0,
            quirk_load_store: true,
            quirk_shift: true,
            vertical_wrapping: true,
        };
        
        // Load fontset
        &cpu.mem[0..CPU::FONTSET.len()].copy_from_slice(CPU::FONTSET);
        
        cpu
    }

    pub fn from_state(state: &[u8]) -> Result<Self, String> {
        Ok(rmp_serde::decode::from_slice(state).map_err(|_| "Failed to deserialize state!")?)
    }

    pub fn save_state(&self) -> Result<Vec<u8>, String> {
        Ok(rmp_serde::encode::to_vec(self).map_err(|_| "Failed to serialize state!")?)
    }

    pub fn load_bootrom(&mut self) {
        self.load_rom(include_bytes!("../../data/roms/pich8-logo.ch8")).unwrap();
    }

    fn load_rom_invalid_opcode(&mut self) {
        self.load_rom(include_bytes!("../../data/roms/pich8-invalid-opcode.ch8")).unwrap();
    }

    pub fn load_rom(&mut self, prog: &[u8]) -> Result<(), String> {
        if prog.len() <= self.mem.len() - 0x200 {
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

    pub fn tick(&mut self, keys: &BitArray<Msb0, [u16; 1]>) {
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
            self.emulate_cycle();
        }
    }

    fn emulate_cycle(&mut self) {
        // Fetch opcode
        self.opcode = (self.mem[self.PC as usize] as u16) << 8 | (self.mem[(self.PC + 1) as usize] as u16);
        
        // Decode opcode
        let x = (self.opcode & 0x0F00) as usize >> 8;
        let y = (self.opcode & 0x00F0) as usize >> 4;
        let nn = (self.opcode & 0x00FF) as u8;
        let nnn = (self.opcode & 0x0FFF) as u16;
        let last_nibble = self.opcode & 0x000F;

        // Execute opcode
        match self.opcode & 0xF000 {
            0x0000 => match nn {
                0xE0 => self.opcode_0x00E0(),
                0xEE => self.opcode_0x00EE(),
                _ => self.opcode_invalid(),
            },
            0x1000 => self.opcode_0x1NNN(nnn),
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
                    0x33 => self.opcode_0xFX33(x),
                    0x55 => self.opcode_0xFX55(x),
                    0x65 => self.opcode_0xFX65(x),
                    _ => self.opcode_invalid(),
            },
            _ => self.opcode_invalid(),
        }
    }

    fn draw_sprite(&mut self, x: usize, y: usize, height: usize) {
        let sprite = &self.mem[self.I as usize..self.I as usize + height];
        let mut collision = false;
        
        for (spr, mut y) in sprite.iter().zip(y..y+height) {
            // Wrap around
            if y >= 32 {
                if self.vertical_wrapping {
                    y %= 32;
                } else {
                    continue;
                }
            }

            for (mut x, i) in (x..x+8).zip((0..8).rev()) {
                // Wrap around
                x %= 64;
                
                let idx = self.get_vmem_index(x, y);
                let bit = (spr >> i) & 0b1 > 0;

                // Detect collision and draw pixel
                if bit && self.vmem[idx] {
                    collision = true;
                }
                let res = self.vmem[idx] != bit;
                self.vmem.set(idx, res);
            }
        }

        self.V[0xF] = collision as u8;
    }

    fn get_vmem_index(&self, x: usize, y: usize) -> usize {
        (y * 64) + x
    }
}

#[cfg(test)]
#[path = "./test.rs"]
mod cpu_test;
