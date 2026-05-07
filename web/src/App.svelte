<script>
  import { onMount, onDestroy } from 'svelte';
  import Screen from './Screen.svelte';
  import RomPicker from './RomPicker.svelte';
  import KeyboardMap from './KeyboardMap.svelte';

  // App state machine: loading | ready | selected | running | paused | halted | error
  let state = 'loading';
  let errorMessage = '';

  let wasm = null;
  let animFrame = null;
  let audioCtx = null;
  let gainNode = null;
  let oscNode = null;

  // Display
  let pixels = new Uint8Array(0);
  let displayWidth = 64;
  let displayHeight = 32;

  // Pressed keys tracking — must be `let` so Svelte reactivity fires on reassign
  let pressedKeys = new Set();

  // Speed control: CHIP-8 cycles executed per animation frame
  let cyclesPerFrame = 10;

  // CHIP-8 key map: physical key → CHIP-8 key index
  const KEYMAP = {
    '1': 0x1, '2': 0x2, '3': 0x3, '4': 0xC,
    'q': 0x4, 'w': 0x5, 'e': 0x6, 'r': 0xD,
    'a': 0x7, 's': 0x8, 'd': 0x9, 'f': 0xE,
    'z': 0xA, 'x': 0x0, 'c': 0xB, 'v': 0xF,
  };

  async function initWasm() {
    try {
      const mod = await import('../pkg/chip8_wasm.js');
      // Call the default export (wasm-bindgen init) to load the .wasm binary
      await mod.default();
      wasm = mod;
      state = 'ready';
    } catch (e) {
      errorMessage = `Failed to load WASM: ${e.message}`;
      state = 'error';
    }
  }

  function initAudio() {
    if (audioCtx) return;
    audioCtx = new AudioContext();
    gainNode = audioCtx.createGain();
    gainNode.gain.setValueAtTime(0, audioCtx.currentTime);
    gainNode.connect(audioCtx.destination);
    oscNode = audioCtx.createOscillator();
    oscNode.type = 'square';
    oscNode.frequency.setValueAtTime(440, audioCtx.currentTime);
    oscNode.connect(gainNode);
    oscNode.start();
  }

  function updateSound(soundTimer) {
    if (!gainNode) return;
    gainNode.gain.setValueAtTime(soundTimer > 0 ? 0.15 : 0, audioCtx.currentTime);
  }

  function loop() {
    if (state !== 'running') return;

    try {
      wasm.step(cyclesPerFrame);
      wasm.tick_timers();
      updateSound(wasm.get_sound_timer());

      displayWidth = wasm.display_width();
      displayHeight = wasm.display_height();
      pixels = wasm.get_display();

      if (wasm.is_halted()) {
        state = 'halted';
        updateSound(0);
        return;
      }
    } catch (e) {
      errorMessage = e.message ?? String(e);
      state = 'error';
      updateSound(0);
      return;
    }

    animFrame = requestAnimationFrame(loop);
  }

  function startLoop() {
    if (animFrame) cancelAnimationFrame(animFrame);
    animFrame = requestAnimationFrame(loop);
  }

  async function loadRom(detail) {
    const { data, filename, mode } = detail;
    initAudio();
    if (audioCtx.state === 'suspended') await audioCtx.resume();

    try {
      wasm.load_rom(data, mode);
    } catch (e) {
      errorMessage = e.message ?? String(e);
      state = 'error';
      return;
    }

    displayWidth = wasm.display_width();
    displayHeight = wasm.display_height();
    pixels = wasm.get_display();
    state = 'running';
    startLoop();
  }

  function togglePause() {
    if (state === 'running') {
      state = 'paused';
      if (animFrame) cancelAnimationFrame(animFrame);
      updateSound(0);
    } else if (state === 'paused') {
      state = 'running';
      startLoop();
    }
  }

  function reset() {
    if (animFrame) cancelAnimationFrame(animFrame);
    updateSound(0);
    wasm.reset();
    pixels = new Uint8Array(0);
    displayWidth = 64;
    displayHeight = 32;
    state = 'ready';
    errorMessage = '';
  }

  async function autoLoadFirstRom() {
    try {
      const manifestRes = await fetch(import.meta.env.BASE_URL + 'roms/manifest.json');
      const roms = await manifestRes.json();
      if (!roms.length) return;
      const first = roms[0];
      const romRes = await fetch(import.meta.env.BASE_URL + 'roms/' + encodeURIComponent(first.file));
      if (!romRes.ok) return;
      const buf = await romRes.arrayBuffer();
      initAudio(); // AudioContext created suspended — no user gesture needed yet
      wasm.load_rom(new Uint8Array(buf), first.mode);
      displayWidth = wasm.display_width();
      displayHeight = wasm.display_height();
      pixels = wasm.get_display();
      state = 'running';
      startLoop();
    } catch {
      // auto-load failed silently — user can pick a ROM manually
    }
  }

  function onKeyDown(e) {
    if (e.repeat) return;
    // Resume audio on first user gesture (autoLoad can't call resume without a gesture)
    if (audioCtx?.state === 'suspended') audioCtx.resume();
    const key = KEYMAP[e.key.toLowerCase()];
    if (key !== undefined && wasm && state !== 'loading') {
      wasm.key_down(key);
      pressedKeys.add(key);
      pressedKeys = pressedKeys; // trigger Svelte reactivity
    }
    if (e.key === ' ') {
      e.preventDefault();
      if (state === 'running' || state === 'paused') togglePause();
    }
  }

  function onKeyUp(e) {
    const key = KEYMAP[e.key.toLowerCase()];
    if (key !== undefined && wasm && state !== 'loading') {
      wasm.key_up(key);
      pressedKeys.delete(key);
      pressedKeys = pressedKeys; // trigger Svelte reactivity
    }
  }

  onMount(async () => {
    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);
    await initWasm();
    autoLoadFirstRom();
  });

  onDestroy(() => {
    if (animFrame) cancelAnimationFrame(animFrame);
    window.removeEventListener('keydown', onKeyDown);
    window.removeEventListener('keyup', onKeyUp);
    oscNode?.stop();
    audioCtx?.close();
  });
</script>

<div class="app">
  <header>
    <h1>Rust (Super) CHIP-8 Emulator</h1>
    <div class="status">
      <span class="dot" class:green={state === 'running'} class:yellow={state === 'paused'} class:red={state === 'error' || state === 'halted'} class:grey={state === 'loading' || state === 'ready' || state === 'selected'}></span>
      <span class="status-text">{state}</span>
    </div>
  </header>

  <main>
    <div class="screen-container">
      <Screen {pixels} {displayWidth} {displayHeight} />

      {#if state === 'error'}
        <div class="overlay error-overlay">
          <p class="error-msg">{errorMessage}</p>
          <button on:click={reset}>Reset</button>
        </div>
      {:else if state === 'halted'}
        <div class="overlay halt-overlay">
          <p>Program halted</p>
          <button on:click={reset}>Reset</button>
        </div>
      {:else if state === 'loading'}
        <div class="overlay loading-overlay">
          <p>Loading WASM…</p>
        </div>
      {/if}
    </div>

    <div class="controls">
      {#if state === 'running' || state === 'paused'}
        <button on:click={togglePause}>{state === 'running' ? 'Pause' : 'Resume'}</button>
        <button on:click={reset}>Reset</button>
      {/if}
      <div class="speed-control">
        <span class="speed-label">Speed</span>
        <input type="range" min="1" max="30" step="1" bind:value={cyclesPerFrame} class="speed-slider" />
        <span class="speed-val">{cyclesPerFrame}×</span>
      </div>
    </div>

    <RomPicker {state} on:load={(e) => loadRom(e.detail)} on:reset={reset} />
    <KeyboardMap {pressedKeys} />
  </main>

  <footer>
    <a href="https://github.com/ShreyRavi/chip8-rs" target="_blank" rel="noopener">chip8-rs</a>
    — CHIP-8 / SCHIP emulator built in Rust, compiled to WebAssembly
  </footer>
</div>

<style>
  :global(*, *::before, *::after) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(body) {
    background: #0a0a0f;
    color: #e0e0e0;
    font-family: 'JetBrains Mono', 'Fira Mono', 'Cascadia Code', monospace;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .app {
    max-width: 720px;
    margin: 0 auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-height: 100vh;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0;
    border-bottom: 1px solid #2a2a3a;
  }

  h1 {
    font-size: 0.9rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: #a0c4ff;
  }

  .status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.75rem;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #444;
    transition: background 0.2s;
  }
  .dot.green  { background: #4ade80; box-shadow: 0 0 6px #4ade80; }
  .dot.yellow { background: #fbbf24; box-shadow: 0 0 6px #fbbf24; }
  .dot.red    { background: #f87171; box-shadow: 0 0 6px #f87171; }
  .dot.grey   { background: #555; }

  main {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    flex: 1;
  }

  .screen-container {
    position: relative;
    width: 100%;
  }

  .overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    backdrop-filter: blur(2px);
    border-radius: 4px;
  }

  .error-overlay { background: rgba(248, 113, 113, 0.15); }
  .halt-overlay  { background: rgba(0, 0, 0, 0.5); }
  .loading-overlay { background: rgba(0, 0, 0, 0.7); }

  .error-msg {
    font-size: 0.8rem;
    color: #f87171;
    max-width: 80%;
    text-align: center;
    word-break: break-word;
  }

  .controls {
    display: flex;
    gap: 0.5rem;
    min-height: 2rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .speed-control {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin-left: auto;
  }

  .speed-label {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: #666;
  }

  .speed-slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100px;
    height: 3px;
    background: #2a2a3e;
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }

  .speed-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #6060a0;
    cursor: pointer;
    transition: background 0.15s;
  }

  .speed-slider::-webkit-slider-thumb:hover {
    background: #a0a0ff;
  }

  .speed-slider::-moz-range-thumb {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #6060a0;
    cursor: pointer;
    border: none;
  }

  .speed-val {
    font-size: 0.75rem;
    color: #888;
    min-width: 2.5rem;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  button {
    padding: 0.4rem 1rem;
    background: #1e1e2e;
    border: 1px solid #3a3a5a;
    border-radius: 4px;
    color: #c0c0e0;
    font-family: inherit;
    font-size: 0.8rem;
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s;
  }

  button:hover {
    background: #2a2a3e;
    border-color: #6060a0;
  }

  footer {
    text-align: center;
    font-size: 0.7rem;
    color: #555;
    padding: 0.5rem 0;
    border-top: 1px solid #2a2a3a;
  }

  footer a {
    color: #7090c0;
    text-decoration: none;
  }

  footer a:hover { text-decoration: underline; }
</style>
