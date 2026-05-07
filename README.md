# chip8-rs

A CHIP-8 and Super-CHIP (SCHIP) emulator written in Rust, compiled to WebAssembly, running in the browser via a Svelte frontend.

**[Play it live:](https://shreyravi.github.io/chip8-rs/)**

---

## Screenshot

```
+-------------------------------------------------------------+
|                                                             |
|              ######  ######  ######                         |
|              ##  ##  ##  ##  ##  ##                         |
|              ######  ######  ######                         |
|              ##  ##  ##  ##  ##  ##                         |
|              ######  ##  ##  ##  ##                         |
|                                                             |
+-------------------------------------------------------------+
  Rust (Super) CHIP-8 Emulator                       * RUNNING
```

---

## What is CHIP-8?

CHIP-8 is an interpreted programming language designed in 1977 by Joseph Weisbecker for the COSMAC VIP microcomputer. Built to simplify game development on early 8-bit hardware, it ran as a virtual machine with 4 KB of memory, a 64x32 monochrome display, 16 general-purpose registers, a simple stack, and a two-tone beeper.

Dozens of games were written for it through the late 1970s and 80s: Pong, Tetris, Space Invaders, Brix. The format got a second life in the 1990s when the Game Boy Color homebrew community extended it into **Super-CHIP (SCHIP)**, adding a 128x64 hi-res mode, scrolling instructions, and larger sprites.

CHIP-8 is the classic starting point for emulator development. The instruction set is small (35 opcodes), the hardware model is well-documented, and every core emulator problem shows up: opcode dispatch, cycle timing, display rendering, audio, input handling, and interpreter quirks that differ between versions.

---

## Features

- **Full CHIP-8 instruction set** - all 35 opcodes implemented
- **Super-CHIP (SCHIP) support** - hi-res 128x64 mode, extended sprites, scroll instructions
- **Compatibility quirks** - per-ROM CHIP-8 vs SCHIP behavior for shift, load/store, and jump
- **WebAssembly runtime** - Rust core compiled with `wasm-pack`, zero JS game logic
- **Audio** - square-wave tone via Web Audio API, driven by the sound timer
- **Auto-start** - CHIP-8 Test ROM plays immediately on page load
- **5 bundled ROMs** - CHIP-8 Test, Pong, Tetris, Brix, Space Invaders (with per-game controls)
- **Custom ROM upload** - load your own `.ch8` / `.c8` / `.rom` file
- **Speed dial** - adjust cycles per frame (1x-30x) live while the emulator runs
- **Keyboard map** - live CHIP-8 keypad visualization with press highlighting
- **74 unit tests** - full opcode coverage in both CHIP-8 and SCHIP modes

---

## Controls

The CHIP-8 keypad is a 4x4 hex grid (0-F), mapped to the left side of a QWERTY keyboard:

```
CHIP-8 Keypad    ->   Physical Keys
+---+---+---+---+    +---+---+---+---+
| 1 | 2 | 3 | C |    | 1 | 2 | 3 | 4 |
| 4 | 5 | 6 | D |    | Q | W | E | R |
| 7 | 8 | 9 | E |    | A | S | D | F |
| A | 0 | B | F |    | Z | X | C | V |
+---+---+---+---+    +---+---+---+---+
```

**Space** pauses and resumes execution.

### Bundled ROM controls

| Game | Controls |
|------|----------|
| CHIP-8 Test | Demo only, no input |
| Pong | P1: `1` up / `Q` down, P2: `4` up / `R` down |
| Tetris | `W` rotate, `E` right, `Q` left, `A` drop |
| Brix | `Q` left, `E` right |
| Space Invaders | `Q` left, `E` right, `W` fire |

---

## Architecture

```
chip8-rs/
+-- crates/
|   +-- chip8-core/          # Pure Rust, no dependencies
|   |   +-- src/
|   |   |   +-- cpu.rs       # Interpreter loop, registers, stack
|   |   |   +-- opcodes.rs   # All 35 opcodes + SCHIP extensions
|   |   |   +-- display.rs   # 64x32 / 128x64 pixel buffer
|   |   |   +-- input.rs     # 16-key state
|   |   |   +-- memory.rs    # 4 KB RAM + font data
|   |   +-- tests/
|   |       +-- opcodes.rs   # 74 unit tests
|   +-- chip8-wasm/          # wasm-bindgen bridge
|       +-- src/lib.rs       # JS-callable API surface
+-- web/                     # Svelte + Vite frontend
    +-- src/
    |   +-- App.svelte        # State machine, audio, animation loop
    |   +-- Screen.svelte     # Canvas renderer
    |   +-- RomPicker.svelte  # ROM selector + file upload
    |   +-- KeyboardMap.svelte
    +-- public/
        +-- roms/             # Bundled ROM files + manifest
```

The emulator core (`chip8-core`) has zero dependencies. No `rand`, no `std::time`, no platform code. A single xorshift PRNG in a `thread_local` cell handles the `RND` opcode. The WASM layer (`chip8-wasm`) exposes a thin API via `wasm-bindgen`. The Svelte frontend drives the `requestAnimationFrame` loop, Web Audio, and keyboard input in JavaScript.

---

## Local Setup

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [Node.js](https://nodejs.org/) 18+ and npm

### Build and run

```bash
# Clone
git clone https://github.com/ShreyRavi/chip8-rs.git
cd chip8-rs

# Run unit tests
cargo test

# Compile Rust to WebAssembly
wasm-pack build crates/chip8-wasm --target web --out-dir ../../web/pkg

# Install frontend deps and start dev server
cd web
npm install
npm run dev
# -> http://localhost:5173
```

### Production build

```bash
cd web
npm run build     # outputs to web/dist/
npm run preview   # serve the production build locally
```

### Run tests

```bash
# All 74 opcode tests
cargo test

# Watch mode (requires cargo-watch)
cargo watch -x test
```

---

## How It Works

### Opcode dispatch

Each instruction is 2 bytes. The interpreter decodes via nibble tuple match:

```rust
match (n1, n2, n3, n4) {
    (0x0, 0x0, 0xE, 0x0) => self.display.clear(),
    (0x1, ..)            => self.pc = nnn,
    (0x6, x, ..)         => self.v[x] = kk,
    (0x8, x, y, 0x4)     => { /* ADD Vx, Vy */ }
    // ...35 opcodes total
}
```

### CHIP-8 vs SCHIP quirks

Three behaviors differ between the original COSMAC VIP interpreter and SCHIP:

| Opcode | CHIP-8 | SCHIP |
|--------|--------|-------|
| `8XY6` / `8XYE` (shift) | shift VY, store in VX | shift VX in place |
| `FX55` / `FX65` (load/store) | increment I | leave I unchanged |
| `BNNN` (jump offset) | V0 + NNN | VX + NNN |

The mode is selected per-ROM at load time.

### Animation loop

The frontend runs `requestAnimationFrame` and executes `cyclesPerFrame` CPU cycles per frame. Default is 10 cycles/frame (~600 Hz at 60 fps), close to the original COSMAC VIP ~500 Hz. The speed dial adjusts this from 1 to 30 live, covering roughly 60 Hz to 1800 Hz. Timers always tick once per frame at 60 Hz regardless of CPU speed - you can adjust it mid game-play!

---

## Deployment

GitHub Actions builds and deploys on every push to `main`:

1. `cargo test` - unit tests must pass
2. `wasm-pack build` - compile Rust to WASM
3. `npm run build` - Vite bundles the frontend (`VITE_BASE=/chip8-rs/`)
4. Deploy to GitHub Pages via `actions/deploy-pages`

---

## License

MIT
