use chip8_core::{Cpu, Mode};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static CHIP8: RefCell<Option<Cpu>> = const { RefCell::new(None) };
}

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn load_rom(data: &[u8], mode: &str) {
    let m = match mode {
        "chip8" => Mode::Chip8,
        "schip" => Mode::Schip,
        m => panic!("unknown mode: {m} (expected 'chip8' or 'schip')"),
    };
    let mut cpu = Cpu::new(m);
    cpu.load_rom(data);
    CHIP8.with(|c| *c.borrow_mut() = Some(cpu));
}

#[wasm_bindgen]
pub fn step(cycles: u32) {
    CHIP8.with(|c| {
        if let Some(cpu) = c.borrow_mut().as_mut() {
            for _ in 0..cycles {
                if cpu.halted { break; }
                cpu.step();
            }
        }
    });
}

#[wasm_bindgen]
pub fn tick_timers() {
    CHIP8.with(|c| {
        if let Some(cpu) = c.borrow_mut().as_mut() {
            cpu.tick_timers();
        }
    });
}

#[wasm_bindgen]
pub fn key_down(key: u8) {
    CHIP8.with(|c| {
        if let Some(cpu) = c.borrow_mut().as_mut() {
            cpu.key_down(key);
        }
    });
}

#[wasm_bindgen]
pub fn key_up(key: u8) {
    CHIP8.with(|c| {
        if let Some(cpu) = c.borrow_mut().as_mut() {
            cpu.key_up(key);
        }
    });
}

#[wasm_bindgen]
pub fn get_display() -> Vec<u8> {
    CHIP8.with(|c| {
        c.borrow().as_ref().map(|cpu| cpu.get_display()).unwrap_or_default()
    })
}

#[wasm_bindgen]
pub fn get_sound_timer() -> u8 {
    CHIP8.with(|c| {
        c.borrow().as_ref().map(|cpu| cpu.sound_timer).unwrap_or(0)
    })
}

#[wasm_bindgen]
pub fn get_delay_timer() -> u8 {
    CHIP8.with(|c| {
        c.borrow().as_ref().map(|cpu| cpu.delay_timer).unwrap_or(0)
    })
}

#[wasm_bindgen]
pub fn is_halted() -> bool {
    CHIP8.with(|c| {
        c.borrow().as_ref().map(|cpu| cpu.halted).unwrap_or(false)
    })
}

#[wasm_bindgen]
pub fn display_width() -> u32 {
    CHIP8.with(|c| {
        c.borrow().as_ref().map(|cpu| cpu.display_width()).unwrap_or(64)
    })
}

#[wasm_bindgen]
pub fn display_height() -> u32 {
    CHIP8.with(|c| {
        c.borrow().as_ref().map(|cpu| cpu.display_height()).unwrap_or(32)
    })
}

#[wasm_bindgen]
pub fn reset() {
    CHIP8.with(|c| *c.borrow_mut() = None);
}
