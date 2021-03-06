use super::*;

#[test]
fn test_initial_state() {
    let cpu = CPU::new();

    assert_eq!(cpu.mem.len(), 65536);
    assert_eq!(cpu.stack, [0; 16]);
    assert_eq!(cpu.keys, [false; 16]);

    assert_eq!(cpu.V, [0; 16]);
    assert_eq!(cpu.I, 0);
    assert_eq!(cpu.PC, 0x200);

    assert_eq!(cpu.sp, 0);

    assert_eq!(cpu.DT, 0);
    assert_eq!(cpu.ST, 0);

    assert_eq!(CPU::FONTSET, &cpu.mem[..CPU::FONTSET.len()]);
}

#[test]
fn test_load_rom() {
    let mut cpu = CPU::new();
    cpu.PC = 0x123;

    let prog: &[u8] = &[
        1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0, 0xF,
    ];
    let _ = cpu.load_rom(prog);

    assert_eq!(&cpu.mem[0x200..0x200 + prog.len()], prog);
    assert_eq!(
        &cpu.mem[0x200 + prog.len()..],
        &[0; 65536][0x200 + prog.len()..]
    );
    assert_eq!(cpu.PC, 0x200);
}

#[test]
fn test_opcodes() {
    // 0x00E0
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x00, 0xE0]);
        cpu.vmem.set_all(true);
        let _ = cpu.emulate_cycle();
        for x in 0..64 {
            for y in 0..32 {
                assert_eq!(cpu.vmem.get_plane(cpu.vmem.current_plane(), x, y), false);
            }
        }
        assert_eq!(cpu.draw, true);
        assert_eq!(cpu.PC, 0x202);
    }

    // 0x00EE
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x00, 0x00, 0x00, 0x00, 0x00, 0xEE]);
        cpu.PC = 0x204;
        cpu.stack[0] = 0x200;
        cpu.sp = 1;
        cpu.prefetch_next_opcode().unwrap();
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x202);
        assert_eq!(cpu.sp, 0);
    }

    // 0x1NNN
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x12, 0xB0]);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x2B0);
        assert_eq!(cpu.sp, 0);
    }

    // 0x2NNN
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x22, 0xB0]);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x2B0);
        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.stack[0], 0x200);
    }

    // 0x3XNN - Equal
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x30, 0x12]);
        cpu.V[0] = 0x12;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x204);
    }
    // 0x3XNN - Not equal
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x30, 0x12]);
        cpu.V[0] = 0x21;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x202);
    }

    // 0x4XNN - Equal
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x40, 0x12]);
        cpu.V[0] = 0x12;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x202);
    }
    // 0x4XNN - Not equal
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x40, 0x12]);
        cpu.V[0] = 0x21;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x204);
    }

    // 0x5XY0 - Equal
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x50, 0x10]);
        cpu.V[0] = 0x12;
        cpu.V[1] = 0x12;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x204);
    }
    // 0x5XY0 - Not equal
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x50, 0x10]);
        cpu.V[0] = 0x21;
        cpu.V[1] = 0x12;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x202);
    }

    // 0x6XNN
    test_arithmetic(0x60AB, 0x11, 0x0, 0xAB, None);

    // 0x7XNN
    test_arithmetic(0x70AB, 0x01, 0x00, 0xAC, None);

    // 0x9XY0 - Equal
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x90, 0x10]);
        cpu.V[0] = 0x12;
        cpu.V[1] = 0x12;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x202);
    }
    // 0x9XY0 - Not equal
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x90, 0x10]);
        cpu.V[0] = 0x21;
        cpu.V[1] = 0x12;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x204);
    }

    // 0xANNN
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xA1, 0x23]);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.I, 0x123);
        assert_eq!(cpu.PC, 0x202);
    }

    // 0xBNNN - Quirk
    {
        let mut cpu = CPU::new();
        cpu.quirk_jump = true;
        let _ = cpu.load_rom(&[0xB1, 0x23]);
        cpu.V[1] = 0x11;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x134);
    }
    // 0xBNNN - No quirk
    {
        let mut cpu = CPU::new();
        cpu.quirk_jump = false;
        let _ = cpu.load_rom(&[0xB1, 0x23]);
        cpu.V[0] = 0x11;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x134);
    }

    // 0xCNNN
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xC0, 0x00]);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.V[0], 0);
        let _ = cpu.load_rom(&[0xC0, 0x0F]);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.V[0] & 0xF0, 0);
        let _ = cpu.load_rom(&[0xC0, 0xF0]);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.V[0] & 0x0F, 0);
        assert_eq!(cpu.PC, 0x202);
    }

    // 0xDXYN - Completely on screen, no clipping/wrapping
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xD0, 0x15]);
        cpu.V[0] = 7;
        cpu.V[1] = 2;
        cpu.I = 0x300;
        cpu.mem[0x300..0x305].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        let _ = cpu.emulate_cycle();
        for y in 2..7 {
            for x in 7..15 {
                assert_eq!(cpu.vmem.get_plane(cpu.vmem.current_plane(), x, y), true);
            }
        }
        assert_eq!(cpu.V[0xF], 0);
        assert_eq!(cpu.draw, true);
        assert_eq!(cpu.PC, 0x202);
    }
    // 0xDXYN - Wrapping when off screen
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xD0, 0x15]);
        cpu.V[0] = 71;
        cpu.V[1] = 34;
        cpu.I = 0x300;
        cpu.mem[0x300..0x305].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        let _ = cpu.emulate_cycle();
        for y in 2..7 {
            for x in 7..15 {
                assert_eq!(cpu.vmem.get_plane(cpu.vmem.current_plane(), x, y), true);
            }
        }
        assert_eq!(cpu.V[0xF], 0);
        assert_eq!(cpu.draw, true);
        assert_eq!(cpu.PC, 0x202);
    }
    // 0xDXYN - Clipping x and y
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xD0, 0x15]);
        cpu.V[0] = 60;
        cpu.V[1] = 30;
        cpu.I = 0x300;
        cpu.mem[0x300..0x305].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        let _ = cpu.emulate_cycle();
        for mut x in 60..68 {
            x %= 64;
            for mut y in 30..35 {
                y %= 32;
                assert_eq!(
                    cpu.vmem.get_plane(cpu.vmem.current_plane(), x, y),
                    x >= 60 && y >= 30
                );
            }
        }
        assert_eq!(cpu.draw, true);
        assert_eq!(cpu.PC, 0x202);
    }
    // 0xDXYN - Wrapping x, but not y
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xD0, 0x15]);
        cpu.quirk_partialwrap_h = true;
        cpu.V[0] = 60;
        cpu.V[1] = 30;
        cpu.I = 0x300;
        cpu.mem[0x300..0x305].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        let _ = cpu.emulate_cycle();
        for mut x in 60..68 {
            x %= 64;
            for mut y in 30..35 {
                y %= 32;
                assert_eq!(cpu.vmem.get_plane(cpu.vmem.current_plane(), x, y), y >= 30);
            }
        }
        assert_eq!(cpu.draw, true);
        assert_eq!(cpu.PC, 0x202);
    }
    // 0xDXYN - Wrapping y, but not x
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xD0, 0x15]);
        cpu.quirk_partialwrap_v = true;
        cpu.V[0] = 60;
        cpu.V[1] = 30;
        cpu.I = 0x300;
        cpu.mem[0x300..0x305].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        let _ = cpu.emulate_cycle();
        for mut x in 60..68 {
            x %= 64;
            for mut y in 30..35 {
                y %= 32;
                assert_eq!(cpu.vmem.get_plane(cpu.vmem.current_plane(), x, y), x >= 60);
            }
        }
        assert_eq!(cpu.draw, true);
        assert_eq!(cpu.PC, 0x202);
    }
    // 0xDXYN - Wrapping x and y
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xD0, 0x15]);
        cpu.quirk_partialwrap_h = true;
        cpu.quirk_partialwrap_v = true;
        cpu.V[0] = 60;
        cpu.V[1] = 30;
        cpu.I = 0x300;
        cpu.mem[0x300..0x305].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        let _ = cpu.emulate_cycle();
        for mut x in 60..68 {
            x %= 64;
            for mut y in 30..35 {
                y %= 32;
                assert_eq!(cpu.vmem.get_plane(cpu.vmem.current_plane(), x, y), true);
            }
        }
        assert_eq!(cpu.draw, true);
        assert_eq!(cpu.PC, 0x202);
    }
    // 0xDXYN - Collision
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xD0, 0x15]);
        cpu.V[0] = 7;
        cpu.V[1] = 2;
        cpu.I = 0x300;
        cpu.mem[0x300..0x305].copy_from_slice(&[0xFF; 5]);
        for x in 7..15 {
            cpu.vmem.set_plane(cpu.vmem.current_plane(), x, 3, true);
        }
        let _ = cpu.emulate_cycle();
        for y in 2..7 {
            for x in 7..15 {
                assert_eq!(cpu.vmem.get_plane(cpu.vmem.current_plane(), x, y), y != 3);
            }
        }
        assert_eq!(cpu.V[0xF], 1);
        assert_eq!(cpu.draw, true);
        assert_eq!(cpu.PC, 0x202);
    }

    // 0xEX9E - Pressed
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xE0, 0x9E]);
        cpu.keys[3] = true;
        cpu.V[0] = 3;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x204);
    }
    // 0xEX9E - Not pressed
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xE0, 0x9E]);
        cpu.keys[3] = false;
        cpu.V[0] = 3;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x202);
    }

    // 0xEXA1 - Pressed
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xE0, 0xA1]);
        cpu.keys[3] = true;
        cpu.V[0] = 3;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x202);
    }
    // 0xEXA1 - Not pressed
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xE0, 0xA1]);
        cpu.keys[3] = false;
        cpu.V[0] = 3;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x204);
    }

    // 0xFX07
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xF0, 0x07]);
        cpu.DT = 0xAB;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.V[0], cpu.DT);
        assert_eq!(cpu.PC, 0x202);
    }

    // 0xFX0A
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xF5, 0x0A]);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.key_wait, true);
        assert_eq!(cpu.key_reg, 5);
    }

    // 0xFX15
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xF0, 0x15]);
        cpu.V[0] = 0x15;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.DT, 0x15);
    }

    // 0xFX18
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xF0, 0x18]);
        cpu.V[0] = 0x15;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.ST, 0x15);
    }

    // 0xFX1E
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xF0, 0x1E]);
        cpu.V[0] = 0x02;
        cpu.I = 0xAB;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.I, 0xAD);
        assert_eq!(cpu.PC, 0x202);
    }

    // 0xFX29
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xF0, 0x29]);
        for i in 0..=0xF {
            cpu.V[0] = i;
            let _ = cpu.emulate_cycle();
            assert_eq!(cpu.I, i as u16 * 5);
            cpu.PC -= 2;
            cpu.prefetch_next_opcode().unwrap();
        }
    }

    // 0xFX33
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xF0, 0x33]);
        cpu.I = 0x300;
        cpu.V[0] = 194;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.mem[cpu.I as usize], 1);
        assert_eq!(cpu.mem[cpu.I as usize + 1], 9);
        assert_eq!(cpu.mem[cpu.I as usize + 2], 4);
    }

    // 0xFX55 - Quirk
    let reg: &[u8] = &[
        0x12, 0x34, 0x56, 0x78, 0x9A, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xF5, 0x55]);
        cpu.I = 0x300;
        cpu.V.copy_from_slice(&reg);
        let _ = cpu.emulate_cycle();
        assert_eq!(&cpu.mem[0x300..=0x305], &reg[..=5]);
        assert_eq!(&cpu.mem[0x306], &0);
        assert_eq!(cpu.PC, 0x202);
        assert_eq!(cpu.I, 0x300);
    }
    // 0xFX55 - No quirk
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xF5, 0x55]);
        cpu.I = 0x300;
        cpu.V.copy_from_slice(&reg);
        cpu.quirk_load_store = false;
        let _ = cpu.emulate_cycle();
        assert_eq!(&cpu.mem[0x300..=0x305], &reg[..=5]);
        assert_eq!(&cpu.mem[0x306], &0);
        assert_eq!(cpu.PC, 0x202);
        assert_eq!(cpu.I, 0x306);
    }

    // 0xFX65 - Quirk
    {
        let prog = &[0xF5, 0x65, 0xA9, 0x87, 0x65, 0x43, 0x21, 0xFF];
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(prog);
        cpu.I = 0x202;
        let _ = cpu.emulate_cycle();
        assert_eq!(&cpu.V[..=5], &prog[2..=7]);
        assert_eq!(&cpu.V[6], &0);
        assert_eq!(cpu.PC, 0x202);
        assert_eq!(cpu.I, 0x202);
    }
    // 0xFX65 - No quirk
    {
        let prog = &[0xF5, 0x65, 0xA9, 0x87, 0x65, 0x43, 0x21, 0xFF];
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(prog);
        cpu.I = 0x202;
        cpu.quirk_load_store = false;
        let _ = cpu.emulate_cycle();
        assert_eq!(&cpu.V[..=5], &prog[2..=7]);
        assert_eq!(&cpu.V[6], &0);
        assert_eq!(cpu.PC, 0x202);
        assert_eq!(cpu.I, 0x208);
    }
}

#[test]
fn test_opcodes_schip() {
    // 0x00FD
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x00, 0xFD]);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x200);
        assert_eq!(&cpu.mem[0x200..0x202], [0x12, 0x00]);
    }

    // 0x00FF & 0x00FE
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x00, 0xFF, 0x00, 0xFE]);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.vmem.video_mode, VideoMode::Extended);
        assert_eq!(cpu.PC, 0x202);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.vmem.video_mode, VideoMode::Default);
        assert_eq!(cpu.PC, 0x204);
    }

    // 0xDXYN
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x00, 0xFF, 0xD0, 0x10]);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x202);
        assert_eq!(cpu.vmem.video_mode, VideoMode::Extended);
        cpu.V[0] = 65;
        cpu.V[1] = 2;
        cpu.I = 0x300;
        cpu.mem[0x300..0x320].copy_from_slice(&[0xFF; 32]);
        let _ = cpu.emulate_cycle();
        for x in 65..81 {
            for y in 2..18 {
                assert_eq!(cpu.vmem.get_plane(cpu.vmem.current_plane(), x, y), true);
            }
        }
        assert_eq!(cpu.V[0xF], 0);
        assert_eq!(cpu.draw, true);
        assert_eq!(cpu.PC, 0x204);
    }

    // 0xFX30
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xF0, 0x30]);
        for i in 0..=9 {
            cpu.V[0] = i;
            let _ = cpu.emulate_cycle();
            assert_eq!(cpu.I, 0x50 + i as u16 * 10);
            cpu.PC -= 2;
            cpu.prefetch_next_opcode().unwrap();
        }
    }

    // 0xFX75
    let reg: &[u8] = &[
        0x12, 0x34, 0x56, 0x78, 0x9A, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xF4, 0x75]);
        cpu.V.copy_from_slice(&reg);
        let _ = cpu.emulate_cycle();
        assert_eq!(&cpu.RPL[..=4], &cpu.V[..=4]);
        assert_eq!(&cpu.RPL[5], &0);
        assert_eq!(cpu.PC, 0x202);
    }

    // 0xFX85
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xF4, 0x85]);
        cpu.RPL[..=5].copy_from_slice(&reg[..=5]);
        let _ = cpu.emulate_cycle();
        assert_eq!(&cpu.RPL[..=4], &cpu.V[..=4]);
        assert_eq!(&cpu.V[5], &0);
        assert_eq!(cpu.PC, 0x202);
    }
}

#[test]
fn test_opcodes_xochip() {
    // 0x5XY2
    let regs = [1, 2, 3, 4, 5];
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x51, 0x52]);
        cpu.V[1..=5].copy_from_slice(&regs);
        cpu.I = 0x300;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.mem[0x300..0x305], regs);
        assert_eq!(cpu.PC, 0x202);
    }
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x58, 0x42]);
        cpu.V[4..=8].copy_from_slice(&regs);
        cpu.I = 0x300;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.mem[0x300..0x305], regs);
        assert_eq!(cpu.PC, 0x202);
    }

    // 0x5XY3
    let regs = [1, 2, 3, 4, 5];
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x51, 0x53]);
        cpu.mem[0x300..0x305].copy_from_slice(&regs);
        cpu.I = 0x300;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.V[1..=5], regs);
        assert_eq!(cpu.PC, 0x202);
    }
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x5F, 0xB3]);
        cpu.mem[0x300..0x305].copy_from_slice(&regs);
        cpu.I = 0x300;
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.V[0xB..=0xF], regs);
        assert_eq!(cpu.PC, 0x202);
    }

    // 0xF000 NNNN
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xF0, 0x00, 0xFE, 0xDC]);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.I, 0xFEDC);
        assert_eq!(cpu.PC, 0x204);
    }

    // 0xFN01
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xF0, 0x01, 0xF1, 0x01, 0xF2, 0x01, 0xF3, 0x01]);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x202);
        assert_eq!(cpu.vmem.current_plane(), Plane::None);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x204);
        assert_eq!(cpu.vmem.current_plane(), Plane::First);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x206);
        assert_eq!(cpu.vmem.current_plane(), Plane::Second);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x208);
        assert_eq!(cpu.vmem.current_plane(), Plane::Both);
    }

    // 0xF002
    {
        let buf = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF];
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0xF0, 0x02]);
        cpu.I = 0x300;
        cpu.mem[0x300..=0x30F].copy_from_slice(&buf);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.audio_buffer, Some(buf));
        assert_eq!(cpu.PC, 0x202);
    }

    // Skip with 4 byte opcode
    {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[0x30, 0x00, 0xF0, 0x00, 0x12, 0x34, 0x12, 0x00]);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x206);
        assert_eq!(cpu.next_opcode, 0x1200);
    }
}

#[test]
fn test_invalid_opcodes() {
    let opcodes = [0u16, 0x5005, 0x9999];
    for opcode in opcodes.iter() {
        let mut cpu = CPU::new();
        let _ = cpu.load_rom(&[(opcode >> 8) as u8, *opcode as u8]);
        let _ = cpu.emulate_cycle();
        assert_eq!(cpu.PC, 0x202);
    }
}

fn test_arithmetic(opcode: u16, v1: u8, v2: u8, res: u8, resv: Option<u8>) {
    let mut cpu = CPU::new();
    let _ = cpu.load_rom(&[(opcode >> 8) as u8, opcode as u8]);
    cpu.V[0] = v1;
    cpu.V[1] = v2;
    let _ = cpu.emulate_cycle();
    assert_eq!(cpu.V[0], res, "Wrong value in V[0]");
    if resv.is_some() {
        assert_eq!(cpu.V[0xF], resv.unwrap(), "Wrong value in V[0xF]");
    }
    assert_eq!(cpu.PC, 0x202);
}

fn test_arithmetic_8(code: u8, v1: u8, v2: u8, res: u8, resv: Option<u8>) {
    assert!(
        code & 0xF0 == 0,
        "Precondition failed: Invalid opcode for arithmetic test!"
    );
    let opcode = 0x8010 | code as u16;
    test_arithmetic(opcode, v1, v2, res, resv);
}

fn test_arithmetic_8_noquirk(code: u8, v1: u8, v2: u8, res: u8, resv: Option<u8>) {
    assert!(
        code & 0xF0 == 0,
        "Precondition failed: Invalid opcode for arithmetic test!"
    );
    let opcode = 0x8010 | code as u16;

    let mut cpu = CPU::new();
    cpu.quirk_load_store = false;
    cpu.quirk_shift = false;
    let _ = cpu.load_rom(&[(opcode >> 8) as u8, opcode as u8]);
    cpu.V[0] = v1;
    cpu.V[1] = v2;
    let _ = cpu.emulate_cycle();
    assert_eq!(cpu.V[0], res, "Wrong value in V[0]");
    if resv.is_some() {
        assert_eq!(cpu.V[0xF], resv.unwrap(), "Wrong value in V[0xF]");
    }
    assert_eq!(cpu.PC, 0x202);
}

#[test]
fn test_opcodes_arithmetic() {
    let b1 = 0b11110010;
    let b2 = 0b00001011;

    // 0x8XY0
    test_arithmetic_8(0, b1, b2, b2, None);

    // 0x8XY1
    test_arithmetic_8(1, b1, b2, b1 | b2, None);

    // 0x8XY2
    test_arithmetic_8(2, b1, b2, b1 & b2, None);

    // 0x8XY3
    test_arithmetic_8(3, b1, b2, b1 ^ b2, None);

    // 0x8XY4 w/ carry
    test_arithmetic_8(4, 0xFF, 1, 0, Some(1));
    // 0x8XY4 w/o carry
    test_arithmetic_8(4, 0xFE, 1, 0xFF, Some(0));

    // 0x8XY5 w/ borrow
    test_arithmetic_8(5, 2, 3, 0xFF, Some(0));
    // 0x8XY5 w/o borrow
    test_arithmetic_8(5, 2, 1, 1, Some(1));

    // 0x8XY6 w/ 0 - Quirk
    test_arithmetic_8(6, b1, b2, b1 >> 1, Some(0));
    // 0x8XY6 w/ 0 - No Quirk
    test_arithmetic_8_noquirk(6, b2, b1, b1 >> 1, Some(0));
    // 0x8XY6 w/ 1 - Quirk
    test_arithmetic_8(6, b2, b1, b2 >> 1, Some(1));
    // 0x8XY6 w/ 1 - No Quirk
    test_arithmetic_8_noquirk(6, b1, b2, b2 >> 1, Some(1));

    // 0x8XY7 w/ borrow
    test_arithmetic_8(7, 2, 3, 1, Some(1));
    // 0x8XY7 w/o borrow
    test_arithmetic_8(7, 3, 2, 0xFF, Some(0));

    // 0x8XYE w/ 0 - Quirk
    test_arithmetic_8(0xE, b1, b2, b1 << 1, Some(1));
    // 0x8XYE w/o 0 - Quirk
    test_arithmetic_8(0xE, b2, b1, b2 << 1, Some(0));
}

#[test]
#[allow(non_snake_case)]
fn test_skipped_opcode_0x0NNN() {
    let mut cpu = CPU::new();
    let _ = cpu.load_rom(&[0x00, 0x00]);
    let _ = cpu.emulate_cycle();
    assert_eq!(cpu.PC, 0x202);
}
