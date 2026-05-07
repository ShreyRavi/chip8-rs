pub const LORES_W: usize = 64;
pub const LORES_H: usize = 32;
pub const HIRES_W: usize = 128;
pub const HIRES_H: usize = 64;

pub struct Display {
    pub pixels: Vec<bool>,
    pub width: usize,
    pub height: usize,
}

impl Display {
    pub fn new() -> Self {
        Display {
            pixels: vec![false; LORES_W * LORES_H],
            width: LORES_W,
            height: LORES_H,
        }
    }

    pub fn set_hires(&mut self, hires: bool) {
        let (w, h) = if hires { (HIRES_W, HIRES_H) } else { (LORES_W, LORES_H) };
        self.width = w;
        self.height = h;
        self.pixels = vec![false; w * h];
    }

    pub fn clear(&mut self) {
        self.pixels.iter_mut().for_each(|p| *p = false);
    }

    /// XOR a sprite at (x, y). Returns true if any pixel was erased.
    pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut collision = false;
        for (row, &byte) in sprite.iter().enumerate() {
            let py = (y + row) % self.height;
            for col in 0..8 {
                if byte & (0x80 >> col) != 0 {
                    let px = (x + col) % self.width;
                    let idx = py * self.width + px;
                    if self.pixels[idx] {
                        collision = true;
                    }
                    self.pixels[idx] ^= true;
                }
            }
        }
        collision
    }

    /// XOR a 16x16 sprite (SCHIP DXY0). Returns true if any pixel was erased.
    pub fn draw_sprite_16(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut collision = false;
        // sprite is 32 bytes: each row is 2 bytes (16 bits wide)
        for row in 0..16 {
            let py = (y + row) % self.height;
            let hi = sprite[row * 2];
            let lo = sprite[row * 2 + 1];
            for col in 0..16usize {
                let bit = if col < 8 {
                    hi & (0x80 >> col) != 0
                } else {
                    lo & (0x80 >> (col - 8)) != 0
                };
                if bit {
                    let px = (x + col) % self.width;
                    let idx = py * self.width + px;
                    if self.pixels[idx] {
                        collision = true;
                    }
                    self.pixels[idx] ^= true;
                }
            }
        }
        collision
    }

    /// Scroll down N lines (SCHIP 00CN).
    pub fn scroll_down(&mut self, n: usize) {
        let w = self.width;
        let h = self.height;
        for row in (0..h).rev() {
            for col in 0..w {
                let dst = row * w + col;
                let src = if row >= n { (row - n) * w + col } else { usize::MAX };
                self.pixels[dst] = if src == usize::MAX { false } else { self.pixels[src] };
            }
        }
    }

    /// Scroll right 4 pixels (SCHIP 00FB).
    pub fn scroll_right(&mut self) {
        let w = self.width;
        let h = self.height;
        for row in 0..h {
            for col in (0..w).rev() {
                let dst = row * w + col;
                self.pixels[dst] = if col >= 4 { self.pixels[row * w + col - 4] } else { false };
            }
        }
    }

    /// Scroll left 4 pixels (SCHIP 00FC).
    pub fn scroll_left(&mut self) {
        let w = self.width;
        let h = self.height;
        for row in 0..h {
            for col in 0..w {
                let dst = row * w + col;
                self.pixels[dst] = if col + 4 < w { self.pixels[row * w + col + 4] } else { false };
            }
        }
    }

    pub fn framebuffer(&self) -> Vec<u8> {
        self.pixels.iter().map(|&p| if p { 1 } else { 0 }).collect()
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}
