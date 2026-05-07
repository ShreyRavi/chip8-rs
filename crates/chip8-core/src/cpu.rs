use crate::display::Display;
use crate::input::Input;
use crate::memory::{Memory, ROM_START};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Chip8,
    Schip,
}

pub struct Cpu {
    pub v: [u8; 16],
    pub i: u16,
    pub pc: u16,
    pub sp: u8,
    pub stack: [u16; 16],
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub halted: bool,
    pub mode: Mode,
    /// Some(reg) means we're waiting for a key press to store in V[reg]
    pub waiting_for_key: Option<u8>,
    pub hp: [u8; 8],
    pub memory: Memory,
    pub display: Display,
    pub input: Input,
}

impl Cpu {
    pub fn new(mode: Mode) -> Self {
        Cpu {
            v: [0u8; 16],
            i: 0,
            pc: ROM_START as u16,
            sp: 0,
            stack: [0u16; 16],
            delay_timer: 0,
            sound_timer: 0,
            halted: false,
            mode,
            waiting_for_key: None,
            hp: [0u8; 8],
            memory: Memory::new(),
            display: Display::new(),
            input: Input::new(),
        }
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        self.memory.load_rom(data);
    }

    /// Execute one CPU cycle. Returns false if halted or waiting.
    pub fn step(&mut self) -> bool {
        if self.halted {
            return false;
        }

        // Handle FX0A key-wait state
        if let Some(reg) = self.waiting_for_key {
            if let Some(key) = self.input.any_pressed() {
                self.v[reg as usize] = key;
                self.waiting_for_key = None;
            } else {
                return false;
            }
        }

        let opcode = self.memory.read_word(self.pc);
        self.pc += 2;
        crate::opcodes::execute(self, opcode);
        true
    }

    /// Decrement timers at 60 Hz. Call once per animation frame.
    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn key_down(&mut self, key: u8) {
        self.input.key_down(key);
    }

    pub fn key_up(&mut self, key: u8) {
        self.input.key_up(key);
    }

    pub fn get_display(&self) -> Vec<u8> {
        self.display.framebuffer()
    }

    pub fn display_width(&self) -> u32 {
        self.display.width as u32
    }

    pub fn display_height(&self) -> u32 {
        self.display.height as u32
    }
}
