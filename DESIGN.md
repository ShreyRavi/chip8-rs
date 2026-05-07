# chip8-rs Design System

Generated from /plan-design-review on 2026-05-07. Update this file when design decisions change.

## Color Tokens

```css
:root {
  --bg: #0d0d0d;           /* page background */
  --surface: #1a1a1a;      /* cassette list, panels */
  --surface2: #242424;     /* hover state */
  --border: #333;          /* dividers, panel borders */
  --phosphor: #33ff66;     /* active pixels, running state indicator */
  --phosphor-dim: #1a8033; /* inactive pixels */
  --amber: #e8800a;        /* CHIP-8 mode accent */
  --cyan: #00e5ff;         /* SCHIP mode accent */
  --text: #c8c8c8;         /* body text */
  --text-dim: #666;        /* labels, secondary info */
  --error: #ff4444;        /* error state */
}
```

## Typography

- **Font:** Space Mono (Google Fonts, `@import` or `<link>`)
- **Weights:** 400 (body) and 700 (game titles, CTAs) — only two weights
- **Letter-spacing:** 0.05–0.15em on uppercase labels (not on body text)
- **Body text minimum:** 12px
- **Labels:** uppercase, 10–11px, letter-spacing 0.1em

## Spacing

- **Base unit:** 8px
- **Panel internal padding:** 14px
- **Page gutters:** 24px
- **Sidebar panel gap:** 20px

## Pixel Rendering (Game Canvas)

- CHIP-8: 64×32 logical pixels → canvas rendered at 8px per logical pixel → 512×256px canvas
- SCHIP: 128×64 logical pixels → canvas at 4px per logical pixel → 512×256px canvas (same physical size)
- Use `image-rendering: pixelated` on the `<canvas>` element
- Pixel gap: draw each cell 7×7px with 1px spacing in canvas `ctx.fillRect()` calls — do NOT use CSS grid for this
- Active pixel glow: `box-shadow: 0 0 4px var(--phosphor)` (CSS-only if using DOM grid; skip if canvas-rendered)
- CRT scanlines: CSS overlay `<div>` with `repeating-linear-gradient(0deg, transparent 2px, rgba(0,0,0,0.15) 2px, rgba(0,0,0,0.15) 4px)` at `pointer-events: none`

## Component Palette

### Cassette (chip8 mode)
- Body border + accent: `var(--amber)` (#e8800a)
- Background tint: `rgba(232,128,10,0.08)`
- Tag: `rgba(232,128,10,0.15)` background, `var(--amber)` text

### Cassette (schip mode)
- Body border + accent: `var(--cyan)` (#00e5ff)
- Background tint: `rgba(0,229,255,0.05)`
- Tag: `rgba(0,229,255,0.1)` background, `var(--cyan)` text

### Buttons
- Default: `border: 1px solid var(--border)`, `color: var(--text)`, transparent bg
- Hover: `border-color: var(--phosphor)`, `color: var(--phosphor)`
- Primary (INSERT CASSETTE): `border-color: var(--phosphor)`, `color: var(--phosphor)`, `font-weight: 700`
- Danger (STOP): `border-color: var(--error)`, `color: var(--error)`
- Border-radius: 2px (all buttons)

### Status Indicators
- Running: 6px dot, `background: var(--phosphor)`, pulsing animation
- Paused: 6px dot, `background: var(--amber)`
- Halted/Error: 6px dot, `background: var(--text-dim)` / `var(--error)`
- Ready: 6px dot, `background: var(--text-dim)`

## Breakpoints

| Breakpoint | Layout |
|---|---|
| ≥1024px | Two-column: `1fr 320px` (screen + sidebar) |
| 640–1023px | Two-column: `1fr 280px` (sidebar narrows) |
| <640px | Single-column stack: header → shelf (horizontal scroll) → screen → controls → footer |

## Accessibility

- Cassette items: `role="button"`, `tabindex="0"`, Enter/Space to select
- INSERT button: `<button>` element, auto-focuses after cassette selection
- Keyboard map toggle: `<button>` with `aria-expanded`, `aria-controls`
- Canvas: `role="img"`, `aria-label="CHIP-8 game display"`
- Contrast ratios: text/bg = 11:1, phosphor/black = 11.5:1, dim text/bg = 3.8:1 (labels only)
- Touch targets: min-height 44px on mobile (cassette items use 52px)

## Interaction States

See "Interaction State Specification" table in the main design doc.

## Eng Review Amendments (2026-05-07)

### Error Handling

All WASM calls in the animation loop wrapped in `try/catch`. On catch: `state = 'error'`, `errorMessage = e.message`.

```js
try {
  step(CYCLES_PER_FRAME);
  tick_timers();
} catch (e) {
  state = 'error';
  errorMessage = e.message;
  cancelAnimationFrame(rafId);
}
```

Add `console_error_panic_hook` to `chip8-wasm/Cargo.toml`. Call `set_once()` in WASM init.

### ROM Delivery

ROMs live in `web/public/roms/` (served as static files, not bundled). Load via:
```js
const url = `${import.meta.env.BASE_URL}roms/${filename}`;
const buf = await fetch(url).then(r => r.arrayBuffer());
```
Remove `assetsInclude: ['**/*.ch8']` from vite.config.

### Buzzer

`OscillatorNode → GainNode → destination`. Created once on INSERT gesture. Toggle per frame:
```js
gain.gain.setValueAtTime(get_sound_timer() > 0 ? 0.3 : 0, ctx.currentTime);
```

### Vite Base URL

```js
// vite.config.js
base: process.env.VITE_BASE ?? '/'
```
GitHub Actions sets `VITE_BASE: '/chip8-rs/'`.

### Custom ROM Upload

File input in `RomPicker.svelte`. User uploads any `.ch8` → appears as CUSTOM cassette in shelf. Mode toggle applies. No persistence (in-memory only).
