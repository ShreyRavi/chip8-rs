pub struct Input {
    pub keys: [bool; 16],
}

impl Input {
    pub fn new() -> Self {
        Input { keys: [false; 16] }
    }

    pub fn key_down(&mut self, key: u8) {
        if key < 16 {
            self.keys[key as usize] = true;
        }
    }

    pub fn key_up(&mut self, key: u8) {
        if key < 16 {
            self.keys[key as usize] = false;
        }
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        key < 16 && self.keys[key as usize]
    }

    pub fn any_pressed(&self) -> Option<u8> {
        self.keys.iter().position(|&k| k).map(|i| i as u8)
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}
