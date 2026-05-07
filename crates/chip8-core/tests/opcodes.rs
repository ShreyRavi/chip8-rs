use chip8_core::{Cpu, Mode};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn chip8() -> Cpu { Cpu::new(Mode::Chip8) }
fn schip() -> Cpu { Cpu::new(Mode::Schip) }

/// Load a 2-byte program and run one step.
fn run(cpu: &mut Cpu, opcode: u16) {
    let bytes = [(opcode >> 8) as u8, (opcode & 0xFF) as u8];
    cpu.load_rom(&bytes);
    cpu.pc = 0x200;
    cpu.step();
}

/// Load a multi-opcode program.
fn rom_from(opcodes: &[u16]) -> Vec<u8> {
    opcodes.iter().flat_map(|&op| [(op >> 8) as u8, (op & 0xFF) as u8]).collect()
}

// ── 00E0 CLS ────────────────────────────────────────────────────────────────

#[test]
fn op_cls_clears_display() {
    let mut cpu = chip8();
    // manually set a pixel
    cpu.display.pixels[0] = true;
    run(&mut cpu, 0x00E0);
    assert!(cpu.display.pixels.iter().all(|&p| !p));
}

// ── 00EE RET ────────────────────────────────────────────────────────────────

#[test]
fn op_ret_pops_stack() {
    let mut cpu = chip8();
    cpu.stack[0] = 0x300;
    cpu.sp = 1;
    run(&mut cpu, 0x00EE);
    assert_eq!(cpu.pc, 0x300);
    assert_eq!(cpu.sp, 0);
}

// ── 1NNN JP ─────────────────────────────────────────────────────────────────

#[test]
fn op_jp_sets_pc() {
    let mut cpu = chip8();
    run(&mut cpu, 0x1ABC);
    assert_eq!(cpu.pc, 0xABC);
}

// ── 2NNN CALL ───────────────────────────────────────────────────────────────

#[test]
fn op_call_pushes_stack() {
    let mut cpu = chip8();
    run(&mut cpu, 0x2300);
    assert_eq!(cpu.pc, 0x300);
    assert_eq!(cpu.sp, 1);
    assert_eq!(cpu.stack[0], 0x202);
}

#[test]
#[should_panic(expected = "stack overflow")]
fn op_call_stack_overflow() {
    let mut cpu = chip8();
    cpu.sp = 16;
    run(&mut cpu, 0x2300);
}

// ── 3XNN SE Vx, byte ────────────────────────────────────────────────────────

#[test]
fn op_se_byte_skip_equal() {
    let mut cpu = chip8();
    cpu.v[0] = 0x42;
    run(&mut cpu, 0x3042);
    assert_eq!(cpu.pc, 0x204); // skipped
}

#[test]
fn op_se_byte_no_skip_ne() {
    let mut cpu = chip8();
    cpu.v[0] = 0x01;
    run(&mut cpu, 0x3042);
    assert_eq!(cpu.pc, 0x202);
}

// ── 4XNN SNE Vx, byte ───────────────────────────────────────────────────────

#[test]
fn op_sne_byte_skip_ne() {
    let mut cpu = chip8();
    cpu.v[0] = 0x01;
    run(&mut cpu, 0x4042);
    assert_eq!(cpu.pc, 0x204);
}

#[test]
fn op_sne_byte_no_skip_equal() {
    let mut cpu = chip8();
    cpu.v[0] = 0x42;
    run(&mut cpu, 0x4042);
    assert_eq!(cpu.pc, 0x202);
}

// ── 5XY0 SE Vx, Vy ──────────────────────────────────────────────────────────

#[test]
fn op_se_reg_skip_equal() {
    let mut cpu = chip8();
    cpu.v[0] = 5; cpu.v[1] = 5;
    run(&mut cpu, 0x5010);
    assert_eq!(cpu.pc, 0x204);
}

#[test]
fn op_se_reg_no_skip_ne() {
    let mut cpu = chip8();
    cpu.v[0] = 5; cpu.v[1] = 6;
    run(&mut cpu, 0x5010);
    assert_eq!(cpu.pc, 0x202);
}

// ── 6XNN LD Vx, byte ────────────────────────────────────────────────────────

#[test]
fn op_ld_byte() {
    let mut cpu = chip8();
    run(&mut cpu, 0x60FF);
    assert_eq!(cpu.v[0], 0xFF);
}

// ── 7XNN ADD Vx, byte ───────────────────────────────────────────────────────

#[test]
fn op_add_byte_no_carry() {
    let mut cpu = chip8();
    cpu.v[0] = 10;
    run(&mut cpu, 0x7005);
    assert_eq!(cpu.v[0], 15);
    assert_eq!(cpu.v[0xF], 0); // no carry flag touch
}

#[test]
fn op_add_byte_wraps() {
    let mut cpu = chip8();
    cpu.v[0] = 0xFF;
    run(&mut cpu, 0x7001);
    assert_eq!(cpu.v[0], 0x00);
}

// ── 8XY0 LD Vx, Vy ──────────────────────────────────────────────────────────

#[test]
fn op_ld_reg() {
    let mut cpu = chip8();
    cpu.v[1] = 0xAB;
    run(&mut cpu, 0x8010);
    assert_eq!(cpu.v[0], 0xAB);
}

// ── 8XY1 OR ─────────────────────────────────────────────────────────────────

#[test]
fn op_or() {
    let mut cpu = chip8();
    cpu.v[0] = 0xF0; cpu.v[1] = 0x0F;
    run(&mut cpu, 0x8011);
    assert_eq!(cpu.v[0], 0xFF);
    assert_eq!(cpu.v[0xF], 0); // VF reset
}

// ── 8XY2 AND ────────────────────────────────────────────────────────────────

#[test]
fn op_and() {
    let mut cpu = chip8();
    cpu.v[0] = 0xFF; cpu.v[1] = 0x0F;
    run(&mut cpu, 0x8012);
    assert_eq!(cpu.v[0], 0x0F);
    assert_eq!(cpu.v[0xF], 0);
}

// ── 8XY3 XOR ────────────────────────────────────────────────────────────────

#[test]
fn op_xor() {
    let mut cpu = chip8();
    cpu.v[0] = 0xFF; cpu.v[1] = 0x0F;
    run(&mut cpu, 0x8013);
    assert_eq!(cpu.v[0], 0xF0);
    assert_eq!(cpu.v[0xF], 0);
}

// ── 8XY4 ADD Vx, Vy ─────────────────────────────────────────────────────────

#[test]
fn op_add_reg_no_carry() {
    let mut cpu = chip8();
    cpu.v[0] = 10; cpu.v[1] = 5;
    run(&mut cpu, 0x8014);
    assert_eq!(cpu.v[0], 15);
    assert_eq!(cpu.v[0xF], 0);
}

#[test]
fn op_add_reg_carry() {
    let mut cpu = chip8();
    cpu.v[0] = 200; cpu.v[1] = 100;
    run(&mut cpu, 0x8014);
    assert_eq!(cpu.v[0], 44); // 300 - 256
    assert_eq!(cpu.v[0xF], 1);
}

// ── 8XY5 SUB Vx, Vy ─────────────────────────────────────────────────────────

#[test]
fn op_sub_no_borrow() {
    let mut cpu = chip8();
    cpu.v[0] = 10; cpu.v[1] = 5;
    run(&mut cpu, 0x8015);
    assert_eq!(cpu.v[0], 5);
    assert_eq!(cpu.v[0xF], 1); // no borrow
}

#[test]
fn op_sub_borrow() {
    let mut cpu = chip8();
    cpu.v[0] = 5; cpu.v[1] = 10;
    run(&mut cpu, 0x8015);
    assert_eq!(cpu.v[0], 251); // wraps
    assert_eq!(cpu.v[0xF], 0); // borrow
}

// ── 8XY6 SHR ────────────────────────────────────────────────────────────────

#[test]
fn op_shr_chip8_reads_vy() {
    let mut cpu = chip8();
    cpu.v[0] = 0xFF; cpu.v[1] = 0b0000_0110;
    run(&mut cpu, 0x8016); // Vx=0, Vy=1
    assert_eq!(cpu.v[0], 0b0000_0011); // Vy >> 1
    assert_eq!(cpu.v[0xF], 0);         // Vy bit0 = 0
}

#[test]
fn op_shr_chip8_flag() {
    let mut cpu = chip8();
    cpu.v[1] = 0b0000_0111;
    run(&mut cpu, 0x8016);
    assert_eq!(cpu.v[0xF], 1); // bit0 of Vy
}

#[test]
fn op_shr_schip_reads_vx() {
    let mut cpu = schip();
    cpu.v[0] = 0b0000_0110; cpu.v[1] = 0xFF;
    run(&mut cpu, 0x8016);
    assert_eq!(cpu.v[0], 0b0000_0011); // Vx >> 1, ignores Vy
    assert_eq!(cpu.v[0xF], 0);
}

// ── 8XY7 SUBN ───────────────────────────────────────────────────────────────

#[test]
fn op_subn_no_borrow() {
    let mut cpu = chip8();
    cpu.v[0] = 5; cpu.v[1] = 10;
    run(&mut cpu, 0x8017); // Vx = Vy - Vx
    assert_eq!(cpu.v[0], 5);
    assert_eq!(cpu.v[0xF], 1);
}

#[test]
fn op_subn_borrow() {
    let mut cpu = chip8();
    cpu.v[0] = 10; cpu.v[1] = 5;
    run(&mut cpu, 0x8017);
    assert_eq!(cpu.v[0xF], 0);
}

// ── 8XYE SHL ────────────────────────────────────────────────────────────────

#[test]
fn op_shl_chip8_reads_vy() {
    let mut cpu = chip8();
    cpu.v[0] = 0; cpu.v[1] = 0b0110_0000;
    run(&mut cpu, 0x801E);
    assert_eq!(cpu.v[0], 0b1100_0000);
    assert_eq!(cpu.v[0xF], 0);
}

#[test]
fn op_shl_chip8_flag() {
    let mut cpu = chip8();
    cpu.v[1] = 0b1110_0000;
    run(&mut cpu, 0x801E);
    assert_eq!(cpu.v[0xF], 1);
}

#[test]
fn op_shl_schip_reads_vx() {
    let mut cpu = schip();
    cpu.v[0] = 0b0110_0000; cpu.v[1] = 0;
    run(&mut cpu, 0x801E);
    assert_eq!(cpu.v[0], 0b1100_0000);
}

// ── 9XY0 SNE Vx, Vy ─────────────────────────────────────────────────────────

#[test]
fn op_sne_reg_skip() {
    let mut cpu = chip8();
    cpu.v[0] = 1; cpu.v[1] = 2;
    run(&mut cpu, 0x9010);
    assert_eq!(cpu.pc, 0x204);
}

#[test]
fn op_sne_reg_no_skip() {
    let mut cpu = chip8();
    cpu.v[0] = 5; cpu.v[1] = 5;
    run(&mut cpu, 0x9010);
    assert_eq!(cpu.pc, 0x202);
}

// ── ANNN LD I, addr ─────────────────────────────────────────────────────────

#[test]
fn op_ld_i() {
    let mut cpu = chip8();
    run(&mut cpu, 0xA123);
    assert_eq!(cpu.i, 0x123);
}

// ── BNNN JP V0/VX ───────────────────────────────────────────────────────────

#[test]
fn op_jp_v0_chip8() {
    let mut cpu = chip8();
    cpu.v[0] = 0x10;
    run(&mut cpu, 0xB100);
    assert_eq!(cpu.pc, 0x110);
}

#[test]
fn op_jp_vx_schip() {
    let mut cpu = schip();
    cpu.v[1] = 0x05;
    run(&mut cpu, 0xB100); // x=1, NNN=0x100
    assert_eq!(cpu.pc, 0x105);
}

// ── CXNN RND ────────────────────────────────────────────────────────────────

#[test]
fn op_rnd_masked() {
    let mut cpu = chip8();
    run(&mut cpu, 0xC00F); // mask 0x0F — result must be 0x00..0x0F
    assert!(cpu.v[0] <= 0x0F);
}

#[test]
fn op_rnd_zero_mask() {
    let mut cpu = chip8();
    run(&mut cpu, 0xC000); // mask 0 → always 0
    assert_eq!(cpu.v[0], 0);
}

// ── DXYN DRW ────────────────────────────────────────────────────────────────

#[test]
fn op_drw_draws_pixels() {
    let mut cpu = chip8();
    // 1-byte sprite at I=0: 0b1111_0000 = 0xF0
    cpu.memory.ram[0x300] = 0xF0;
    cpu.i = 0x300;
    cpu.v[0] = 0; cpu.v[1] = 0;
    run(&mut cpu, 0xD011); // draw at (0,0), 1 row
    // Pixels 0-3 should be lit, 4-7 dark
    assert!(cpu.display.pixels[0]);
    assert!(cpu.display.pixels[1]);
    assert!(cpu.display.pixels[2]);
    assert!(cpu.display.pixels[3]);
    assert!(!cpu.display.pixels[4]);
    assert_eq!(cpu.v[0xF], 0); // no collision on blank screen
}

#[test]
fn op_drw_collision() {
    let mut cpu = chip8();
    cpu.memory.ram[0x300] = 0xFF;
    cpu.i = 0x300;
    // Pre-light a pixel
    cpu.display.pixels[0] = true;
    cpu.v[0] = 0; cpu.v[1] = 0;
    run(&mut cpu, 0xD011);
    assert_eq!(cpu.v[0xF], 1);
}

#[test]
fn op_drw_wraps_horizontal() {
    let mut cpu = chip8();
    cpu.memory.ram[0x300] = 0xFF;
    cpu.i = 0x300;
    cpu.v[0] = 62; cpu.v[1] = 0; // near right edge
    run(&mut cpu, 0xD011);
    // should wrap — no panic
    assert_eq!(cpu.v[0xF], 0);
}

// ── EX9E SKP / EXA1 SKNP ────────────────────────────────────────────────────

#[test]
fn op_skp_pressed() {
    let mut cpu = chip8();
    cpu.v[0] = 0x5;
    cpu.input.keys[0x5] = true;
    run(&mut cpu, 0xE09E);
    assert_eq!(cpu.pc, 0x204);
}

#[test]
fn op_skp_not_pressed() {
    let mut cpu = chip8();
    cpu.v[0] = 0x5;
    run(&mut cpu, 0xE09E);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn op_sknp_not_pressed() {
    let mut cpu = chip8();
    cpu.v[0] = 0x5;
    run(&mut cpu, 0xE0A1);
    assert_eq!(cpu.pc, 0x204);
}

#[test]
fn op_sknp_pressed() {
    let mut cpu = chip8();
    cpu.v[0] = 0x5;
    cpu.input.keys[0x5] = true;
    run(&mut cpu, 0xE0A1);
    assert_eq!(cpu.pc, 0x202);
}

// ── FX07 / FX15 / FX18 timers ───────────────────────────────────────────────

#[test]
fn op_ld_dt_to_vx() {
    let mut cpu = chip8();
    cpu.delay_timer = 0x30;
    run(&mut cpu, 0xF007);
    assert_eq!(cpu.v[0], 0x30);
}

#[test]
fn op_ld_vx_to_dt() {
    let mut cpu = chip8();
    cpu.v[0] = 0x20;
    run(&mut cpu, 0xF015);
    assert_eq!(cpu.delay_timer, 0x20);
}

#[test]
fn op_ld_vx_to_st() {
    let mut cpu = chip8();
    cpu.v[0] = 0x10;
    run(&mut cpu, 0xF018);
    assert_eq!(cpu.sound_timer, 0x10);
}

#[test]
fn tick_timers_decrements() {
    let mut cpu = chip8();
    cpu.delay_timer = 5;
    cpu.sound_timer = 3;
    cpu.tick_timers();
    assert_eq!(cpu.delay_timer, 4);
    assert_eq!(cpu.sound_timer, 2);
}

#[test]
fn tick_timers_no_underflow() {
    let mut cpu = chip8();
    cpu.delay_timer = 0;
    cpu.sound_timer = 0;
    cpu.tick_timers();
    assert_eq!(cpu.delay_timer, 0);
    assert_eq!(cpu.sound_timer, 0);
}

// ── FX0A key wait ───────────────────────────────────────────────────────────

#[test]
fn op_key_wait_sets_state() {
    let mut cpu = chip8();
    run(&mut cpu, 0xF00A);
    assert_eq!(cpu.waiting_for_key, Some(0));
}

#[test]
fn op_key_wait_resumes_on_press() {
    let mut cpu = chip8();
    let rom = rom_from(&[0xF00A, 0x0000]);
    cpu.load_rom(&rom);
    cpu.pc = 0x200;
    cpu.step(); // starts waiting
    assert!(cpu.waiting_for_key.is_some());
    cpu.step(); // still waiting — no key
    assert!(cpu.waiting_for_key.is_some());
    cpu.input.keys[0x7] = true;
    cpu.step(); // key pressed — resumes
    assert!(cpu.waiting_for_key.is_none());
    assert_eq!(cpu.v[0], 0x7);
}

// ── FX1E ADD I, Vx ──────────────────────────────────────────────────────────

#[test]
fn op_add_i() {
    let mut cpu = chip8();
    cpu.i = 0x100; cpu.v[0] = 0x10;
    run(&mut cpu, 0xF01E);
    assert_eq!(cpu.i, 0x110);
}

// ── FX29 font ───────────────────────────────────────────────────────────────

#[test]
fn op_ld_font() {
    let mut cpu = chip8();
    cpu.v[0] = 0xA;
    run(&mut cpu, 0xF029);
    assert_eq!(cpu.i, 0 + 0xA * 5); // FONT_ADDR + digit * 5
}

// ── FX33 BCD ────────────────────────────────────────────────────────────────

#[test]
fn op_bcd() {
    let mut cpu = chip8();
    cpu.v[0] = 234;
    cpu.i = 0x300;
    run(&mut cpu, 0xF033);
    assert_eq!(cpu.memory.ram[0x300], 2);
    assert_eq!(cpu.memory.ram[0x301], 3);
    assert_eq!(cpu.memory.ram[0x302], 4);
}

// ── FX55 / FX65 store/load (with I quirk) ───────────────────────────────────

#[test]
fn op_store_chip8_increments_i() {
    let mut cpu = chip8();
    cpu.v[0] = 0xAA; cpu.v[1] = 0xBB; cpu.v[2] = 0xCC;
    cpu.i = 0x300;
    run(&mut cpu, 0xF255); // store V0..V2
    assert_eq!(cpu.memory.ram[0x300], 0xAA);
    assert_eq!(cpu.memory.ram[0x301], 0xBB);
    assert_eq!(cpu.memory.ram[0x302], 0xCC);
    assert_eq!(cpu.i, 0x303); // incremented
}

#[test]
fn op_store_schip_no_increment() {
    let mut cpu = schip();
    cpu.v[0] = 0xAA; cpu.v[1] = 0xBB;
    cpu.i = 0x300;
    run(&mut cpu, 0xF155);
    assert_eq!(cpu.i, 0x300); // unchanged
}

#[test]
fn op_load_chip8_increments_i() {
    let mut cpu = chip8();
    cpu.memory.ram[0x300] = 0x11;
    cpu.memory.ram[0x301] = 0x22;
    cpu.i = 0x300;
    run(&mut cpu, 0xF165);
    assert_eq!(cpu.v[0], 0x11);
    assert_eq!(cpu.v[1], 0x22);
    assert_eq!(cpu.i, 0x302);
}

#[test]
fn op_load_schip_no_increment() {
    let mut cpu = schip();
    cpu.memory.ram[0x300] = 0x11;
    cpu.i = 0x300;
    run(&mut cpu, 0xF065);
    assert_eq!(cpu.i, 0x300);
}

// ── SCHIP: 00CN scroll down ──────────────────────────────────────────────────

#[test]
fn op_schip_scroll_down() {
    let mut cpu = schip();
    cpu.display.pixels[0] = true; // top-left pixel set
    run(&mut cpu, 0x00C2); // scroll down 2
    assert!(!cpu.display.pixels[0]); // moved down
    assert!(cpu.display.pixels[2 * cpu.display.width]); // 2 rows down
}

// ── SCHIP: 00FB/00FC scroll right/left ──────────────────────────────────────

#[test]
fn op_schip_scroll_right() {
    let mut cpu = schip();
    cpu.display.pixels[0] = true;
    run(&mut cpu, 0x00FB);
    assert!(!cpu.display.pixels[0]);
    assert!(cpu.display.pixels[4]);
}

#[test]
fn op_schip_scroll_left() {
    let mut cpu = schip();
    cpu.display.pixels[4] = true;
    run(&mut cpu, 0x00FC);
    assert!(!cpu.display.pixels[4]);
    assert!(cpu.display.pixels[0]);
}

// ── SCHIP: 00FD EXIT ────────────────────────────────────────────────────────

#[test]
fn op_schip_exit() {
    let mut cpu = schip();
    run(&mut cpu, 0x00FD);
    assert!(cpu.halted);
}

// ── SCHIP: 00FE/00FF hires ──────────────────────────────────────────────────

#[test]
fn op_schip_hires() {
    let mut cpu = schip();
    run(&mut cpu, 0x00FF);
    assert_eq!(cpu.display.width, 128);
    assert_eq!(cpu.display.height, 64);
}

#[test]
fn op_schip_lores() {
    let mut cpu = schip();
    cpu.display.set_hires(true);
    run(&mut cpu, 0x00FE);
    assert_eq!(cpu.display.width, 64);
    assert_eq!(cpu.display.height, 32);
}

// ── SCHIP: FX75/FX85 HP flags ───────────────────────────────────────────────

#[test]
fn op_schip_store_hp() {
    let mut cpu = schip();
    cpu.v[0] = 0x11; cpu.v[1] = 0x22;
    run(&mut cpu, 0xF175); // store V0..V1
    assert_eq!(cpu.hp[0], 0x11);
    assert_eq!(cpu.hp[1], 0x22);
}

#[test]
fn op_schip_load_hp() {
    let mut cpu = schip();
    cpu.hp[0] = 0x33; cpu.hp[1] = 0x44;
    run(&mut cpu, 0xF185); // load V0..V1
    assert_eq!(cpu.v[0], 0x33);
    assert_eq!(cpu.v[1], 0x44);
}

// ── SCHIP: DXY0 16×16 sprite ────────────────────────────────────────────────

#[test]
fn op_schip_drw_16x16() {
    let mut cpu = schip();
    cpu.display.set_hires(true);
    // 32 bytes all 0xFF = solid 16x16 block
    for i in 0..32 {
        cpu.memory.ram[0x300 + i] = 0xFF;
    }
    cpu.i = 0x300;
    cpu.v[0] = 0; cpu.v[1] = 0;
    run(&mut cpu, 0xD010); // DXY0 = 16x16
    // All 16 pixels in row 0 should be lit
    for col in 0..16 {
        assert!(cpu.display.pixels[col], "pixel {col} not set");
    }
    assert_eq!(cpu.v[0xF], 0);
}

// ── SCHIP: FX30 large font ───────────────────────────────────────────────────

#[test]
fn op_schip_large_font() {
    let mut cpu = schip();
    cpu.v[0] = 5;
    run(&mut cpu, 0xF030);
    assert_eq!(cpu.i, 0x050 + 5 * 10); // LARGE_FONT_ADDR + digit * 10
}

// ── Memory direct tests ──────────────────────────────────────────────────────

#[test]
fn memory_font_loaded() {
    let cpu = chip8();
    // Digit 0 font sprite first byte = 0xF0
    assert_eq!(cpu.memory.ram[0], 0xF0);
}

#[test]
fn memory_load_rom_too_large() {
    use chip8_core::cpu::Cpu;
    let mut cpu = Cpu::new(Mode::Chip8);
    let result = std::panic::catch_unwind(move || {
        cpu.load_rom(&vec![0u8; 4096]);
    });
    assert!(result.is_err());
}

// ── Input ────────────────────────────────────────────────────────────────────

#[test]
fn input_key_down_up() {
    use chip8_core::input::Input;
    let mut inp = Input::new();
    inp.key_down(0x5);
    assert!(inp.is_pressed(0x5));
    inp.key_up(0x5);
    assert!(!inp.is_pressed(0x5));
}

#[test]
fn input_any_pressed() {
    use chip8_core::input::Input;
    let mut inp = Input::new();
    assert_eq!(inp.any_pressed(), None);
    inp.key_down(0xA);
    assert_eq!(inp.any_pressed(), Some(0xA));
}

// ── Display ──────────────────────────────────────────────────────────────────

#[test]
fn display_framebuffer_length() {
    use chip8_core::display::Display;
    let d = Display::new();
    assert_eq!(d.framebuffer().len(), 64 * 32);
}

#[test]
fn display_scroll_down_empty_rows() {
    use chip8_core::display::Display;
    let mut d = Display::new();
    d.pixels[0] = true;
    d.scroll_down(1);
    assert!(!d.pixels[0]);
    assert!(d.pixels[64]); // moved down one row
}
