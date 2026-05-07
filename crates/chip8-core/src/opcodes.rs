use crate::cpu::{Cpu, Mode};
use crate::memory::Memory;

pub fn execute(cpu: &mut Cpu, opcode: u16) {
    let n1 = ((opcode & 0xF000) >> 12) as u8;
    let n2 = ((opcode & 0x0F00) >> 8) as u8;  // x
    let n3 = ((opcode & 0x00F0) >> 4) as u8;  // y
    let n4 = (opcode & 0x000F) as u8;         // n
    let x = n2 as usize;
    let y = n3 as usize;
    let nnn = opcode & 0x0FFF;
    let kk = (opcode & 0x00FF) as u8;

    match (n1, n2, n3, n4) {
        // 00CN — SCHIP scroll down N lines
        (0x0, 0x0, 0xC, n) if cpu.mode == Mode::Schip => {
            cpu.display.scroll_down(n as usize);
        }

        // 00E0 — CLS
        (0x0, 0x0, 0xE, 0x0) => {
            cpu.display.clear();
        }

        // 00EE — RET
        (0x0, 0x0, 0xE, 0xE) => {
            assert!(cpu.sp > 0, "CHIP-8 stack underflow: RET on empty stack");
            cpu.sp -= 1;
            cpu.pc = cpu.stack[cpu.sp as usize];
        }

        // 00FB — SCHIP scroll right 4 pixels
        (0x0, 0x0, 0xF, 0xB) if cpu.mode == Mode::Schip => {
            cpu.display.scroll_right();
        }

        // 00FC — SCHIP scroll left 4 pixels
        (0x0, 0x0, 0xF, 0xC) if cpu.mode == Mode::Schip => {
            cpu.display.scroll_left();
        }

        // 00FD — SCHIP EXIT
        (0x0, 0x0, 0xF, 0xD) if cpu.mode == Mode::Schip => {
            cpu.halted = true;
        }

        // 00FE — SCHIP LOW (disable hires)
        (0x0, 0x0, 0xF, 0xE) if cpu.mode == Mode::Schip => {
            cpu.display.set_hires(false);
        }

        // 00FF — SCHIP HIGH (enable hires)
        (0x0, 0x0, 0xF, 0xF) if cpu.mode == Mode::Schip => {
            cpu.display.set_hires(true);
        }

        // 0NNN — SYS (no-op in modern interpreters)
        (0x0, _, _, _) => {}

        // 1NNN — JP addr
        (0x1, _, _, _) => {
            cpu.pc = nnn;
        }

        // 2NNN — CALL addr
        (0x2, _, _, _) => {
            assert!(cpu.sp < 16, "CHIP-8 stack overflow (16-level limit)");
            cpu.stack[cpu.sp as usize] = cpu.pc;
            cpu.sp += 1;
            cpu.pc = nnn;
        }

        // 3XNN — SE Vx, byte
        (0x3, _, _, _) => {
            if cpu.v[x] == kk {
                cpu.pc += 2;
            }
        }

        // 4XNN — SNE Vx, byte
        (0x4, _, _, _) => {
            if cpu.v[x] != kk {
                cpu.pc += 2;
            }
        }

        // 5XY0 — SE Vx, Vy
        (0x5, _, _, 0x0) => {
            if cpu.v[x] == cpu.v[y] {
                cpu.pc += 2;
            }
        }

        // 6XNN — LD Vx, byte
        (0x6, _, _, _) => {
            cpu.v[x] = kk;
        }

        // 7XNN — ADD Vx, byte (no carry flag)
        (0x7, _, _, _) => {
            cpu.v[x] = cpu.v[x].wrapping_add(kk);
        }

        // 8XY0 — LD Vx, Vy
        (0x8, _, _, 0x0) => {
            cpu.v[x] = cpu.v[y];
        }

        // 8XY1 — OR Vx, Vy
        (0x8, _, _, 0x1) => {
            cpu.v[x] |= cpu.v[y];
            cpu.v[0xF] = 0; // Octo-compatible: reset VF
        }

        // 8XY2 — AND Vx, Vy
        (0x8, _, _, 0x2) => {
            cpu.v[x] &= cpu.v[y];
            cpu.v[0xF] = 0;
        }

        // 8XY3 — XOR Vx, Vy
        (0x8, _, _, 0x3) => {
            cpu.v[x] ^= cpu.v[y];
            cpu.v[0xF] = 0;
        }

        // 8XY4 — ADD Vx, Vy (carry in VF)
        (0x8, _, _, 0x4) => {
            let (res, carry) = cpu.v[x].overflowing_add(cpu.v[y]);
            cpu.v[x] = res;
            cpu.v[0xF] = carry as u8;
        }

        // 8XY5 — SUB Vx, Vy (VF = NOT borrow)
        (0x8, _, _, 0x5) => {
            let (res, borrow) = cpu.v[x].overflowing_sub(cpu.v[y]);
            cpu.v[x] = res;
            cpu.v[0xF] = (!borrow) as u8;
        }

        // 8XY6 — SHR Vx {, Vy}
        // CHIP-8 (Octo): Vx = Vy >> 1, VF = Vy & 1
        // SCHIP: Vx = Vx >> 1, VF = Vx & 1
        (0x8, _, _, 0x6) => {
            let src = if cpu.mode == Mode::Chip8 { cpu.v[y] } else { cpu.v[x] };
            let flag = src & 0x1;
            cpu.v[x] = src >> 1;
            cpu.v[0xF] = flag;
        }

        // 8XY7 — SUBN Vx, Vy (VF = NOT borrow)
        (0x8, _, _, 0x7) => {
            let (res, borrow) = cpu.v[y].overflowing_sub(cpu.v[x]);
            cpu.v[x] = res;
            cpu.v[0xF] = (!borrow) as u8;
        }

        // 8XYE — SHL Vx {, Vy}
        // CHIP-8 (Octo): Vx = Vy << 1, VF = Vy >> 7
        // SCHIP: Vx = Vx << 1, VF = Vx >> 7
        (0x8, _, _, 0xE) => {
            let src = if cpu.mode == Mode::Chip8 { cpu.v[y] } else { cpu.v[x] };
            let flag = (src >> 7) & 0x1;
            cpu.v[x] = src << 1;
            cpu.v[0xF] = flag;
        }

        // 9XY0 — SNE Vx, Vy
        (0x9, _, _, 0x0) => {
            if cpu.v[x] != cpu.v[y] {
                cpu.pc += 2;
            }
        }

        // ANNN — LD I, addr
        (0xA, _, _, _) => {
            cpu.i = nnn;
        }

        // BNNN — JP V0, addr
        // CHIP-8: PC = V0 + NNN
        // SCHIP: PC = VX + NNN
        (0xB, _, _, _) => {
            let base = if cpu.mode == Mode::Chip8 { cpu.v[0] } else { cpu.v[x] };
            cpu.pc = nnn.wrapping_add(base as u16);
        }

        // CXNN — RND Vx, byte
        (0xC, _, _, _) => {
            let rnd = rand_byte();
            cpu.v[x] = rnd & kk;
        }

        // DXYN — DRW Vx, Vy, nibble
        (0xD, _, _, 0x0) if cpu.mode == Mode::Schip => {
            // 16x16 sprite
            let sx = cpu.v[x] as usize;
            let sy = cpu.v[y] as usize;
            let sprite_len = 32; // 16 rows × 2 bytes
            let addr = cpu.i as usize;
            let sprite = &cpu.memory.ram[addr..addr + sprite_len];
            let collision = cpu.display.draw_sprite_16(sx, sy, sprite);
            cpu.v[0xF] = collision as u8;
        }

        (0xD, _, _, n) => {
            let sx = cpu.v[x] as usize;
            let sy = cpu.v[y] as usize;
            let addr = cpu.i as usize;
            let sprite = &cpu.memory.ram[addr..addr + n as usize];
            let collision = cpu.display.draw_sprite(sx, sy, sprite);
            cpu.v[0xF] = collision as u8;
        }

        // EX9E — SKP Vx
        (0xE, _, 0x9, 0xE) => {
            if cpu.input.is_pressed(cpu.v[x]) {
                cpu.pc += 2;
            }
        }

        // EXA1 — SKNP Vx
        (0xE, _, 0xA, 0x1) => {
            if !cpu.input.is_pressed(cpu.v[x]) {
                cpu.pc += 2;
            }
        }

        // FX07 — LD Vx, DT
        (0xF, _, 0x0, 0x7) => {
            cpu.v[x] = cpu.delay_timer;
        }

        // FX0A — LD Vx, K (wait for key press)
        (0xF, _, 0x0, 0xA) => {
            cpu.waiting_for_key = Some(x as u8);
        }

        // FX15 — LD DT, Vx
        (0xF, _, 0x1, 0x5) => {
            cpu.delay_timer = cpu.v[x];
        }

        // FX18 — LD ST, Vx
        (0xF, _, 0x1, 0x8) => {
            cpu.sound_timer = cpu.v[x];
        }

        // FX1E — ADD I, Vx
        (0xF, _, 0x1, 0xE) => {
            cpu.i = cpu.i.wrapping_add(cpu.v[x] as u16);
        }

        // FX29 — LD F, Vx (set I to font sprite for digit Vx)
        (0xF, _, 0x2, 0x9) => {
            cpu.i = Memory::font_addr(cpu.v[x] & 0xF);
        }

        // FX30 — SCHIP LD HF, Vx (large font)
        (0xF, _, 0x3, 0x0) if cpu.mode == Mode::Schip => {
            cpu.i = Memory::large_font_addr(cpu.v[x] & 0xF);
        }

        // FX33 — LD B, Vx (BCD)
        (0xF, _, 0x3, 0x3) => {
            let val = cpu.v[x];
            cpu.memory.write_byte(cpu.i, val / 100);
            cpu.memory.write_byte(cpu.i + 1, (val / 10) % 10);
            cpu.memory.write_byte(cpu.i + 2, val % 10);
        }

        // FX55 — LD [I], Vx (store V0..Vx in memory)
        // CHIP-8: I increments after each store
        // SCHIP: I unchanged
        (0xF, _, 0x5, 0x5) => {
            for reg in 0..=x {
                cpu.memory.write_byte(cpu.i + reg as u16, cpu.v[reg]);
            }
            if cpu.mode == Mode::Chip8 {
                cpu.i += x as u16 + 1;
            }
        }

        // FX65 — LD Vx, [I] (load V0..Vx from memory)
        // CHIP-8: I increments after each load
        // SCHIP: I unchanged
        (0xF, _, 0x6, 0x5) => {
            for reg in 0..=x {
                cpu.v[reg] = cpu.memory.read_byte(cpu.i + reg as u16);
            }
            if cpu.mode == Mode::Chip8 {
                cpu.i += x as u16 + 1;
            }
        }

        // FX75 — SCHIP STOR (store V0..Vx in HP flags, max 7)
        (0xF, _, 0x7, 0x5) if cpu.mode == Mode::Schip => {
            let count = x.min(7);
            for reg in 0..=count {
                cpu.hp[reg] = cpu.v[reg];
            }
        }

        // FX85 — SCHIP LOAD (load V0..Vx from HP flags, max 7)
        (0xF, _, 0x8, 0x5) if cpu.mode == Mode::Schip => {
            let count = x.min(7);
            for reg in 0..=count {
                cpu.v[reg] = cpu.hp[reg];
            }
        }

        _ => {
            // Unknown opcode — no-op (some ROMs use undocumented instructions)
        }
    }
}

/// Simple xorshift PRNG — avoids pulling in rand crate.
fn rand_byte() -> u8 {
    use std::cell::Cell;
    thread_local! {
        static STATE: Cell<u32> = const { Cell::new(0xDEAD_BEEF) };
    }
    STATE.with(|s| {
        let mut x = s.get();
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        s.set(x);
        (x & 0xFF) as u8
    })
}
