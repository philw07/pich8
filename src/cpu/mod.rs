use std::fmt;
use rand::prelude::*;
use bitvec::prelude::*;
use getset::{CopyGetters, Getters, Setters};
use serde::{Serialize, Deserialize};
use crate::video_memory::{VideoMemory, VideoMode, Plane};

mod opcodes;

#[derive(Debug)]
pub enum Error {
    SaveStateError(rmp_serde::encode::Error),
    LoadStateError(rmp_serde::decode::Error),
    ProgramCounterOverflow,
    StackOverflow,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Error::SaveStateError(e) => write!(f, "Save state error: {}", e),
            Error::LoadStateError(e) => write!(f, "Load state error: {}", e),
            Error::ProgramCounterOverflow => write!(f, "Program counter overflow!"),
            Error::StackOverflow => write!(f, "Stack overflow occurred! The ROM might be invalid or different quirk settings required."),
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

pub enum Breakpoint {
    PC(u16),
    I(u16),
    Opcode(String),
}

#[allow(non_snake_case)]
#[derive(CopyGetters, Getters, Setters, Serialize, Deserialize)]
pub struct CPU {
    mem: Box<[u8]>,                         // Main memory
    #[getset(get = "pub")]
    vmem: VideoMemory,                      // Graphics memory
    #[getset(get_copy = "pub")]
    stack: [u16; 16],                       // Stack to store locations before a jump occurs
    keys: BitArray<Msb0, [u16; 1]>,         // Keypad status
    #[getset(get = "pub")]
    audio_buffer: Option<[u8; 16]>,         // XO-CHIP audio buffer

    #[getset(get_copy = "pub")]
    PC: u16,                                // Program counter
    #[getset(get_copy = "pub")]
    V: [u8; 16],                            // Registers
    #[getset(get_copy = "pub")]
    I: u16,                                 // Index register
    #[getset(get_copy = "pub")]
    DT: u8,                                 // Delay timer
    #[getset(get_copy = "pub")]
    ST: u8,                                 // Sound timer
    #[getset(get_copy = "pub")]
    RPL: [u8; 8],                           // HP48 RPL flags (used for S-CHIP)

    #[getset(get_copy = "pub")]
    opcode: u16,                            // Current opcode
    #[getset(get = "pub")]
    opcode_description: String,             // Current opcode description
    #[getset(get_copy = "pub")]
    next_opcode: u16,                       // Next opcode
    #[getset(get = "pub")]
    next_opcode_description: String,        // Next opcode description
    next_opcode_ext: u16,                   // Next opcode extension in case of 32bit opcode (XO-CHIP)
    #[getset(get_copy = "pub")]
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
            mem: vec![0; u16::MAX as usize + 1].into_boxed_slice(),
            vmem: VideoMemory::new(),
            stack: [0; 16],
            keys: bitarr![Msb0, u16; 0; 16],
            audio_buffer: None,

            PC: CPU::PC_INITIAL,
            V: [0; 16],
            I: 0,
            DT: 0,
            ST: 0,
            RPL: [0; 8],

            opcode: 0,
            opcode_description: String::new(),
            next_opcode: 0,
            next_opcode_description: String::new(),
            next_opcode_ext: 0,
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
            self.prefetch_next_opcode().map_err(|e| format!("{}", e))
        } else {
            self.load_bootrom();
            Err("Invalid ROM!".to_string())
        }
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

    pub fn check_breakpoint(&self, breakpoint: Breakpoint) -> bool {
        match breakpoint {
            Breakpoint::PC(val) => self.PC == val,
            Breakpoint::I(val) => self.I == val,
            Breakpoint::Opcode(pattern) => {
                if pattern.len() != 4 {
                    false
                } else {
                    let val = format!("{:04X}", self.next_opcode);
                    for (pat_c, val_c) in pattern.chars().zip(val.chars()) {
                        if pat_c != '*' && pat_c != val_c {
                            return false;
                        }
                    }
                    true
                }
            },
        }
    }

    fn prefetch_next_opcode(&mut self) -> Result<(), Error> {
        if self.PC as usize >= self.mem.len() - 2 {
            self.load_bootrom();
            return Err(Error::ProgramCounterOverflow);
        }
        self.next_opcode = (self.mem[self.PC as usize] as u16) << 8 | (self.mem[self.PC as usize + 1] as u16);
        self.next_opcode_description = self.get_next_opcode_description();
        if self.next_opcode == 0xF000 {
            self.next_opcode_ext = (self.mem[self.PC as usize + 2] as u16) << 8 | (self.mem[self.PC as usize + 3] as u16);
        }
        Ok(())
    }

    fn emulate_cycle(&mut self) -> Result<(), Error> {
        // Fetch opcode
        self.opcode = self.next_opcode;
        self.opcode_description = self.next_opcode_description.clone();

        // Decode opcode
        let h = (self.opcode & 0xF000) as usize >> 12;
        let x = (self.opcode & 0x0F00) as usize >> 8;
        let y = (self.opcode & 0x00F0) as usize >> 4;
        let n = (self.opcode & 0x000F) as u8;
        let nn = (self.opcode & 0x00FF) as u8;
        let nnn = (self.opcode & 0x0FFF) as u16;

        // Execute opcode
        match (h, x, y, n) {
            (  0,   0, 0xC, _  ) => self.opcode_schip_0x00CN(n),
            (  0,   0, 0xD, _  ) => self.opcode_xochip_0x00DN(n),
            (  0,   0, 0xE,   0) => self.opcode_0x00E0(),
            (  0,   0, 0xE, 0xE) => self.opcode_0x00EE(),
            (  0,   0, 0xF, 0xB) => self.opcode_schip_0x00FB(),
            (  0,   0, 0xF, 0xC) => self.opcode_schip_0x00FC(),
            (  0,   0, 0xF, 0xD) => self.opcode_schip_0x00FD(),
            (  0,   0, 0xF, 0xE) => self.opcode_schip_0x00FE(),
            (  0,   0, 0xF, 0xF) => self.opcode_schip_0x00FF(),
            (  0,   2,   3,   0) => self.opcode_hires_0x0230(),

            (  1,   2,   6,   0) => self.opcode_0x1260(nnn),
            (  1, _  , _  , _  ) => self.opcode_0x1NNN(nnn),

            (  2, _  , _  , _  ) => self.opcode_0x2NNN(nnn)?,

            (  3, _  , _  , _  ) => self.opcode_0x3XNN(x, nn),

            (  4, _  , _  , _  ) => self.opcode_0x4XNN(x, nn),

            (  5, _  , _  ,   0) => self.opcode_0x5XY0(x, y),
            (  5, _  , _  ,   2) => self.opcode_xochip_0x5XY2(x, y),
            (  5, _  , _  ,   3) => self.opcode_xochip_0x5XY3(x, y),

            (  6, _  , _  , _  ) => self.opcode_0x6XNN(x, nn),

            (  7, _  , _  , _  ) => self.opcode_0x7XNN(x, nn),

            (  8, _  , _  ,   0) => self.opcode_0x8XY0(x, y),
            (  8, _  , _  ,   1) => self.opcode_0x8XY1(x, y),
            (  8, _  , _  ,   2) => self.opcode_0x8XY2(x, y),
            (  8, _  , _  ,   3) => self.opcode_0x8XY3(x, y),
            (  8, _  , _  ,   4) => self.opcode_0x8XY4(x, y),
            (  8, _  , _  ,   5) => self.opcode_0x8XY5(x, y),
            (  8, _  , _  ,   6) => self.opcode_0x8XY6(x, y),
            (  8, _  , _  ,   7) => self.opcode_0x8XY7(x, y),
            (  8, _  , _  , 0xE) => self.opcode_0x8XYE(x, y),

            (  9, _  , _  ,   0) => self.opcode_0x9XY0(x, y),

            (0xA, _  , _  , _  ) => self.opcode_0xANNN(nnn),

            (0xB, _  , _  , _  ) => self.opcode_0xBNNN(nnn),

            (0xC, _  , _  , _  ) => self.opcode_0xCXNN(x, nn),

            (0xD, _  , _  , _  ) => self.opcode_0xDXYN(x, y, n as usize),

            (0xE, _  ,   9, 0xE) => self.opcode_0xEX9E(x),
            (0xE, _  , 0xA,   1) => self.opcode_0xEXA1(x),

            (0xF,   0,   0,   0) => self.opcode_xochip_0xF000(),
            (0xF, _  ,   0,   1) => self.opcode_xochip_0xFN01(x),
            (0xF,   0,   0,   2) => self.opcode_xochip_0xF002(),
            (0xF, _  ,   0,   7) => self.opcode_0xFX07(x),
            (0xF, _  ,   0, 0xA) => self.opcode_0xFX0A(x),
            (0xF, _  ,   1,   5) => self.opcode_0xFX15(x),
            (0xF, _  ,   1,   8) => self.opcode_0xFX18(x),
            (0xF, _  ,   1, 0xE) => self.opcode_0xFX1E(x),
            (0xF, _  ,   2,   9) => self.opcode_0xFX29(x),
            (0xF, _  ,   3,   0) => self.opcode_schip_0xFX30(x),
            (0xF, _  ,   3,   3) => self.opcode_0xFX33(x),
            (0xF, _  ,   5,   5) => self.opcode_0xFX55(x),
            (0xF, _  ,   6,   5) => self.opcode_0xFX65(x),
            (0xF, _  ,   7,   5) => self.opcode_schip_0xFX75(x),
            (0xF, _  ,   8,   5) => self.opcode_schip_0xFX85(x),

            _                    => self.opcode_invalid(),
        }

        // Fetch next opcode
        self.prefetch_next_opcode()
    }

    fn draw_sprite(&mut self, x: usize, y: usize, height: usize) {
        let big_sprite = (self.vmem.video_mode() == &VideoMode::Extended || self.quirk_draw) && height == 0;
        let step = if big_sprite { 2 } else { 1 };
        let width = if big_sprite { 16 } else { 8 };
        let height = if height == 0 { 16 } else { height };

        let mut collision = false;
        let mut i = self.I as usize;
        let len = width/8 * height;

        for plane in vec![Plane::First, Plane::Second] {
            if self.vmem.current_plane() == plane || self.vmem.current_plane() == Plane::Both {
                let sprite = &self.mem[i..i + len];
                i += len;
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
                        if bit && self.vmem.get_plane(plane, x, y) {
                            collision = true;
                        }
                        let res = self.vmem.get_plane(plane, x, y) != bit;
                        self.vmem.set_plane(plane, x, y, res);
                    }
                }
            }
        }

        self.V[0xF] = collision as u8;
    }

    fn get_next_opcode_description(&self, ) -> String {
        let h = (self.next_opcode & 0xF000) as usize >> 12;
        let x = (self.next_opcode & 0x0F00) as usize >> 8;
        let y = (self.next_opcode & 0x00F0) as usize >> 4;
        let n = (self.next_opcode & 0x000F) as u8;
        let nn = (self.next_opcode & 0x00FF) as u8;
        let nnn = (self.next_opcode & 0x0FFF) as u16;
    

        match (h, x, y, n) {
            (  0,   0, 0xC, _  ) => format!("SCD {}", n),
            (  0,   0, 0xD, _  ) => String::from("SCU [XO-CHIP]"),
            (  0,   0, 0xE,   0) => String::from("CLS"),
            (  0,   0, 0xE, 0xE) => String::from("RET"),
            (  0,   0, 0xF, 0xB) => String::from("SCR [S-CHIP]"),
            (  0,   0, 0xF, 0xC) => String::from("SCL [S-CHIP]"),
            (  0,   0, 0xF, 0xD) => String::from("EXIT [S-CHIP]"),
            (  0,   0, 0xF, 0xE) => String::from("LOW [S-CHIP]"),
            (  0,   0, 0xF, 0xF) => String::from("HIGH [S-CHIP]"),
            (  0,   2,   3,   0) => if self.vmem.video_mode() == &VideoMode::HiRes { String::from("CLS [HiRes]") } else { format!("SYS {:03X} (Ignored)", nnn) },

            (  1,   2,   6,   0) => if self.PC == 0x200 { String::from("HIRES [HiRes]") } else { format!("JP {:03X}", nnn) },
            (  1, _  , _  , _  ) => format!("JP {:03X}", nnn),

            (  2, _  , _  , _  ) => format!("CALL {:03X}", nnn),

            (  3, _  , _  , _  ) => format!("SE V{:X} ({:02X}), {:02X}", x, self.V[x], nn),

            (  4, _  , _  , _  ) => format!("SNE V{:X} ({:02X}), {:02X}", x, self.V[x], nn),

            (  5, _  , _  ,   0) => format!("SE V{:X} ({:02X}), V{:X} ({:02X})", x, self.V[x], y, self.V[y]),
            (  5, _  , _  ,   2) => format!("LD [I], V{:X}, V{:X} [XO-CHIP]", x, y),
            (  5, _  , _  ,   3) => format!("LD V{:X}, V{:X}, [I] [XO-CHIP]", x, y),

            (  6, _  , _  , _  ) => format!("LD V{:X}, {:02X}", x, nn),

            (  7, _  , _  , _  ) => format!("ADD V{:X} ({:02X}), {:02X}", x, self.V[x], nn),

            (  8, _  , _  ,   0) => format!("LD V{:X}, V{:X} ({:02X})", x, y, self.V[y]),
            (  8, _  , _  ,   1) => format!("OR V{:X} ({:02X}), V{:X} ({:02X})", x, self.V[x], y, self.V[y]),
            (  8, _  , _  ,   2) => format!("AND V{:X} ({:02X}), V{:X} ({:02X})", x, self.V[x], y, self.V[y]),
            (  8, _  , _  ,   3) => format!("XOR V{:X} ({:02X}), V{:X} ({:02X})", x, self.V[x], y, self.V[y]),
            (  8, _  , _  ,   4) => format!("ADD V{:X} ({:02X}), V{:X} ({:02X})", x, self.V[x], y, self.V[y]),
            (  8, _  , _  ,   5) => format!("SUB V{:X} ({:02X}), V{:X} ({:02X})", x, self.V[x], y, self.V[y]),
            (  8, _  , _  ,   6) => if self.quirk_shift { format!("SHR V{:X} ({:02X})", x, self.V[x]) }
                                    else { format!("SHR V{:X}, V{:X} ({:02X})", x, y, self.V[y]) },
            (  8, _  , _  ,   7) => format!("SUBN V{:X} ({:02X}), V{:X} ({:02X})", x, self.V[x], y, self.V[y]),
            (  8, _  , _  , 0xE) => if self.quirk_shift { format!("SHL V{:X} ({:02X})", x, self.V[x]) }
                                    else { format!("SHL V{:X}, V{:X} ({:02X})", x, y, self.V[y]) },

            (  9, _  , _  ,   0) => format!("SNE V{:X} ({:02X}), V{:X} ({:02X})", x, self.V[x], y, self.V[y]),

            (0xA, _  , _  , _  ) => format!("LD I, {:03X}", nnn),

            (0xB, _  , _  , _  ) => if self.quirk_jump { format!("JP V{:X} ({:03X}), {:03X}", x, self.V[x], nnn) }
                                    else { format!("JP V0 ({:02X}), {:03X}", self.V[0], nnn) },

            (0xC, _  , _  , _  ) => format!("RND V{:X}, {:02X}", x, nn),

            (0xD, _  , _  , _  ) => format!("DRW V{:X} ({:02X}), V{:X} ({:02X}), {:X}", x, self.V[x], y, self.V[y], n),

            (0xE, _  ,   9, 0xE) => format!("SKP V{:X} ({:02X})", x, self.V[x]),
            (0xE, _  , 0xA,   1) => format!("SKNP V{:X} ({:02X})", x, self.V[x]),

            (0xF,   0,   0,   0) => format!("LD I, {:04X} [XO-CHIP]", self.next_opcode_ext),
            (0xF, _  ,   0,   1) => format!("PLANE {} [XO-CHIP]", x),
            (0xF,   0,   0,   2) => String::from("AUDIO [XO-CHIP]"),
            (0xF, _  ,   0,   7) => format!("LD V{:X}, DT ({:02X})", x, self.DT),
            (0xF, _  ,   0, 0xA) => format!("LD V{:X}, K", x),
            (0xF, _  ,   1,   5) => format!("LD DT, V{:X} ({:02X})", x, self.V[x]),
            (0xF, _  ,   1,   8) => format!("LD ST, V{:X} ({:02X})", x, self.V[x]),
            (0xF, _  ,   1, 0xE) => format!("ADD I, V{:X} ({:02X})", x, self.V[x]),
            (0xF, _  ,   2,   9) => format!("LD F, V{:X} ({:02X})", x, self.V[x]),
            (0xF, _  ,   3,   0) => format!("LD F, V{:X} ({:02X}) [S-CHIP]", x, self.V[x]),
            (0xF, _  ,   3,   3) => format!("LD B, V{:X} ({:02X})", x, self.V[x]),
            (0xF, _  ,   5,   5) => format!("LD [I], V{:X}", x),
            (0xF, _  ,   6,   5) => format!("LD V{:X}, [I]", x),
            (0xF, _  ,   7,   5) => format!("LD R, V{:X} [S-CHIP]", x),
            (0xF, _  ,   8,   5) => format!("LD V{:X}, R [S-CHIP]", x),

            _                    => String::from("Invalid"),
        }
    }
}

#[cfg(test)]
#[path = "./test.rs"]
mod cpu_test;
