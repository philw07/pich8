use super::*;

#[allow(non_snake_case)]
impl CPU {
    // Invalid ipcode
    #[inline]
    pub(super) fn opcode_invalid(&mut self) {
        self.PC += 2;
    }

    // 0x00CN - SCHIP - Scroll display N lines down
    #[inline]
    pub(super) fn opcode_schip_0x00CN(&mut self, n: u8) {
        self.vmem.scroll_down(n as usize);
        self.PC += 2;
    }

    // 0x00E0 - Clear display
    #[inline]
    pub(super) fn opcode_0x00E0(&mut self) {
        self.vmem.clear();
        self.draw = true;
        self.PC += 2;
    }

    // 0x00EE - Return from subroutine
    #[inline]
    pub(super) fn opcode_0x00EE(&mut self) {
        self.sp -= 1;
        self.PC = self.stack[self.sp] + 2;
    }

    // 0x00FB - SCHIP - Scroll display 4 pixels right
    #[inline]
    pub(super) fn opcode_schip_0x00FB(&mut self) {
        self.vmem.scroll_right();
        self.PC += 2;
    }

    // 0x00FC - SCHIP - Scroll display 4 pixels left
    #[inline]
    pub(super) fn opcode_schip_0x00FC(&mut self) {
        self.vmem.scroll_left();
        self.PC += 2;
    }

    // 0x00FD - SCHIP - Exit interpreter
    #[inline]
    pub(super) fn opcode_schip_0x00FD(&mut self) {
        // Instead of actually exiting, we're creating an endless loop
        self.mem[0x200..0x202].copy_from_slice(&[0x12, 0x00]);
        self.PC = 0x200;
    }

    // 0x00FE - SCHIP - Disable extended screen mode
    #[inline]
    pub(super) fn opcode_schip_0x00FE(&mut self) {
        self.vmem.set_video_mode(VideoMode::Default);
        self.PC += 2;
    }

    // 0x00FF - SCHIP - Enable extended screen mode
    #[inline]
    pub(super) fn opcode_schip_0x00FF(&mut self) {
        self.vmem.set_video_mode(VideoMode::Extended);
        self.PC += 2;
    }

    // 0x0230 - HiRes - Clear screen
    #[inline]
    pub(super) fn opcode_hires_0x0230(&mut self) {
        if self.vmem.video_mode() == &VideoMode::HiRes {
            self.vmem.clear();
            self.draw = true;
            self.PC += 2;
        } else {
            self.opcode_0x0NNN();
        }
    }

    // 0x0NNN - Legacy SYS call, ignored
    #[inline]
    pub(super) fn opcode_0x0NNN(&mut self) {
        self.PC += 2;
    }

    // 0x1NNN - Goto nnn
    #[inline]
    pub(super) fn opcode_0x1NNN(&mut self, nnn: u16) {
        self.PC = nnn;
    }

    // 0x1260 - Activate HiRes mode - only if it's the first opcode
    #[inline]
    pub(super) fn opcode_0x1260(&mut self, nnn: u16) {
        if self.PC == 0x200 {
            self.vmem.set_video_mode(VideoMode::HiRes);
            self.PC = 0x2C0;
        } else {
            self.opcode_0x1NNN(nnn);
        }
    }

    // 0x2NNN - Call subroutine at nnn
    #[inline]
    pub(super) fn opcode_0x2NNN(&mut self, nnn: u16) {
        self.stack[self.sp] = self.PC;
        self.sp += 1;
        self.PC = nnn;
    }

    // 0x3XNN - Skip next instruction if Vx == nn
    #[inline]
    pub(super) fn opcode_0x3XNN(&mut self, x: usize, nn: u8) {
        self.PC += if self.V[x] == nn { 4 } else { 2 };
    }

    // 0x4XNN - Skip next instruction if Vx != nn
    #[inline]
    pub(super) fn opcode_0x4XNN(&mut self, x: usize, nn: u8) {
        self.PC += if self.V[x] != nn { 4 } else { 2 };
    }

    // 0x5XY0 - Skip next instruction if Vx == Vy
    #[inline]
    pub(super) fn opcode_0x5XY0(&mut self, x: usize, y: usize) {
        self.PC += if self.V[x] == self.V[y] { 4 } else { 2 };
    }

    // 0x6XNN - Vx = nn
    #[inline]
    pub(super) fn opcode_0x6XNN(&mut self, x: usize, nn: u8) {
        self.V[x] = nn;
        self.PC += 2;
    }

    // 0x7XNN - Vx += nn
    #[inline]
    pub(super) fn opcode_0x7XNN(&mut self, x: usize, nn: u8) {
        let res = self.V[x] as u16 + nn as u16;
        self.V[x] = res as u8;
        self.PC += 2;
    }

    // 0x8XY0 - Vx = Vy
    #[inline]
    pub(super) fn opcode_0x8XY0(&mut self, x: usize, y: usize) {
        self.V[x] = self.V[y];
        self.PC += 2;
    }

    // 0x8XY1 - Vx |= Vy
    #[inline]
    pub(super) fn opcode_0x8XY1(&mut self, x: usize, y: usize) {
        self.V[x] |= self.V[y];
        self.PC += 2;
    }

    // 0x8XY2 - Vx &= Vy
    #[inline]
    pub(super) fn opcode_0x8XY2(&mut self, x: usize, y: usize) {
        self.V[x] &= self.V[y];
        self.PC += 2;
    }

    // 0x8XY3 - Vx ^= Vy
    #[inline]
    pub(super) fn opcode_0x8XY3(&mut self, x: usize, y: usize) {
        self.V[x] ^= self.V[y];
        self.PC += 2;
    }

    // 0x8XY4 - Vx += Vy
    #[inline]
    pub(super) fn opcode_0x8XY4(&mut self, x: usize, y: usize) {
        let res = self.V[x] as u16 + self.V[y] as u16;
        let vf = if res > 0xFF { 1 } else { 0 };
        self.write_vf(x, res as u8, vf);
        self.PC += 2;
    }

    // 0x8XY5 - Vx -= Vy
    #[inline]
    pub(super) fn opcode_0x8XY5(&mut self, x: usize, y: usize) {
        let res = self.V[x] as i16 - self.V[y] as i16;
        let vf = if res < 0 { 0 } else { 1 };
        self.write_vf(x, res as u8, vf);
        self.PC += 2;
    }

    // 0x8XY6 - Bitshift right
    // Original: Vx = Vy >> 1
    // Quirk:    Vx >>= 1
    #[inline]
    pub(super) fn opcode_0x8XY6(&mut self, x: usize, y: usize) {
        if self.quirk_shift {
            self.write_vf(x, self.V[x] >> 1, self.V[x] & 1);
        } else {
            self.write_vf(x, self.V[y] >> 1, self.V[y] & 1);
        }
        self.PC += 2;
    }

    // 0x8XY7 - Vx = Vy - Vx
    #[inline]
    pub(super) fn opcode_0x8XY7(&mut self, x: usize, y: usize) {
        let res = self.V[y] as i16 - self.V[x] as i16;
        let vf = if res < 0 { 0 } else { 1 };
        self.write_vf(x, res as u8, vf);
        self.PC += 2;
    }

    // 0x8XYE - Bitshift left
    // Original: Vx = Vy << 1
    // Quirk:    Vx <<= 1
    #[inline]
    pub(super) fn opcode_0x8XYE(&mut self, x: usize, y: usize) {
        if self.quirk_shift {
            self.write_vf(x, self.V[x] << 1, (self.V[x] & 0x80) >> 7);
        } else {
            self.write_vf(x, self.V[y] << 1, (self.V[y] & 0x80) >> 7);
        }
        self.PC += 2;
    }

    // 0x9XY0 - Skip next instruction if Vx != Vy
    #[inline]
    pub(super) fn opcode_0x9XY0(&mut self, x: usize, y: usize) {
        self.PC += if self.V[x] != self.V[y] { 4 } else { 2 };
    }

    // 0xANNN - I = nnn
    #[inline]
    pub(super) fn opcode_0xANNN(&mut self, nnn: u16) {
        self.I = nnn;
        self.PC += 2;
    }

    // 0xBNNN - PC = nnn + V0
    // Original: PC = nnn + V0
    // Quirk:    PC = xnn + Vx
    #[inline]
    pub(super) fn opcode_0xBNNN(&mut self, nnn: u16) {
        self.PC = nnn + if self.quirk_jump { self.V[(nnn >> 8 & 0xF) as usize] } else { self.V[0] } as u16;
    }

    // 0xCXNN - Vx = rand() & nn
    #[inline]
    pub(super) fn opcode_0xCXNN(&mut self, x: usize, nn: u8) {
        let mut rng = rand::thread_rng();
        self.V[x] = rng.gen::<u8>() & nn;
        self.PC += 2;
    }

    // 0xDXYN - draw(Vx, Vy, n)
    #[inline]
    pub(super) fn opcode_0xDXYN(&mut self, x: usize, y: usize, n: usize) {
        self.draw_sprite(self.V[x] as usize, self.V[y] as usize, n);
        self.draw = true;
        self.PC += 2;
    }

    // 0xEX9E - Skip next instruction if key(Vx) is pressed
    #[inline]
    pub(super) fn opcode_0xEX9E(&mut self, x: usize) {
        self.PC += if self.keys[self.V[x] as usize] { 4 } else { 2 };
    }

    // 0xEXA1 - Skip next instruction if key(Vx) is not pressed
    #[inline]
    pub(super) fn opcode_0xEXA1(&mut self, x: usize) {
        self.PC += if !self.keys[self.V[x] as usize] { 4 } else { 2 };
    }

    // 0xFX07 - Vx = DT
    #[inline]
    pub(super) fn opcode_0xFX07(&mut self, x: usize) {
        self.V[x] = self.DT;
        self.PC += 2;
    }

    // 0xFX0A - Vx = get_key();
    #[inline]
    pub(super) fn opcode_0xFX0A(&mut self, x: usize) {
        self.key_wait = true;
        self.key_reg = x;
        self.PC += 2;
    }

    // 0xFX15 - DT = Vx
    #[inline]
    pub(super) fn opcode_0xFX15(&mut self, x: usize) {
        self.DT = self.V[x];
        self.PC += 2;
    }

    // 0xFX18 - ST = Vx
    #[inline]
    pub(super) fn opcode_0xFX18(&mut self, x: usize) {
        self.ST = self.V[x];
        self.PC += 2;
    }

    // 0xFX1E - I += Vx
    #[inline]
    pub(super) fn opcode_0xFX1E(&mut self, x: usize) {
        // I was quite puzzled as the ROM Spacefight2091! wasn't working correctly and weird graphics issues occurred.
        // After some research, I found the answer using the test ROM "SCTEST" (Description: https://github.com/Rubenknex/Chip-8/blob/master/sctest.txt).
        // The error that occured is "Error 24" which describes an undocumented feature in this opcode setting VF to 1 if I overflows.
        self.I += self.V[x] as u16;
        if self.I as usize >= self.mem.len() {
            self.V[0xF] = 1;
        }
        self.PC += 2;
    }

    // 0xFX29 - I = sprite_add(Vx)
    #[inline]
    pub(super) fn opcode_0xFX29(&mut self, x: usize) {
        self.I = self.V[x] as u16 * 5;
        self.PC += 2;
    }

    // 0xFX30 - SCHIP - I = 10-byte sprite_add(Vx)
    #[inline]
    pub(super) fn opcode_schip_0xFX30(&mut self, x: usize) {
        self.I = 0x50 + self.V[x] as u16 * 10;
        self.PC += 2;
    }

    // 0xFX33 - set_BCD(Vx)
    #[inline]
    pub(super) fn opcode_0xFX33(&mut self, x: usize) {
        let hundreds = self.V[x] / 100;
        let tens = (self.V[x] % 100) / 10;
        let ones = self.V[x] % 10;
        self.mem[self.I as usize] = hundreds;
        self.mem[self.I as usize + 1] = tens;
        self.mem[self.I as usize + 2] = ones;
        self.PC += 2;
    }

    // 0xFX55 - reg_dump(Vx, &I)
    // Original: I is incremented
    // Quirk:    I is not incremented
    #[inline]
    pub(super) fn opcode_0xFX55(&mut self, x: usize) {
        let start = self.I as usize;
        let end = self.I as usize + x;
        self.mem[start..=end].copy_from_slice(&self.V[..=x]);
        if !self.quirk_load_store {
            self.I += x as u16 + 1;
        }
        self.PC += 2;
    }

    // 0xFX65 - reg_load(Vx, &I)
    // Original: I is incremented
    // Quirk:    I is not incremented
    #[inline]
    pub(super) fn opcode_0xFX65(&mut self, x: usize) {
        let start = self.I as usize;
        let end = self.I as usize + x;
        self.V[..=x].copy_from_slice(&self.mem[start..=end]);
        if !self.quirk_load_store {
            self.I += x as u16 + 1;
        }
        self.PC += 2;
    }

    // 0xFX75 - SCHIP - Store V0..VX in RPL user flags (X < 8)
    #[inline]
    pub(super) fn opcode_schip_0xFX75(&mut self, x: usize) {
        self.RPL[..=x].copy_from_slice(&self.V[..=x]);
        self.PC += 2;
    }

    // 0xFX85 - SCHIP - Read V0..VX from RPL user flags (X < 8)
    #[inline]
    pub(super) fn opcode_schip_0xFX85(&mut self, x: usize) {
        self.V[..=x].copy_from_slice(&self.RPL[..=x]);
        self.PC += 2;
    }

    fn write_vf(&mut self, reg: usize, value: u8, vf: u8) {
        if self.quirk_vf_order {
            self.V[reg] = value;
            self.V[0xF] = vf;
        } else {
            self.V[0xF] = vf;
            self.V[reg] = value;
        }
    }
}
