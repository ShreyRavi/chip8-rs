<script>
  import { onMount, createEventDispatcher } from 'svelte';

  export let state;

  const dispatch = createEventDispatcher();

  let roms = [];
  let selectedRom = null;
  let selectedMode = 'chip8';
  let loadError = '';
  let fileInput;

  onMount(async () => {
    try {
      const res = await fetch(import.meta.env.BASE_URL + 'roms/manifest.json');
      roms = await res.json();
    } catch {
      roms = [];
    }
  });

  async function loadSelected() {
    if (!selectedRom) return;
    loadError = '';
    try {
      const res = await fetch(import.meta.env.BASE_URL + 'roms/' + encodeURIComponent(selectedRom.file));
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const buf = await res.arrayBuffer();
      dispatch('load', { data: new Uint8Array(buf), filename: selectedRom.file, mode: selectedMode });
    } catch (e) {
      loadError = e.message;
    }
  }

  async function onFileChange(e) {
    const file = e.target.files?.[0];
    if (!file) return;
    loadError = '';
    try {
      const buf = await file.arrayBuffer();
      dispatch('load', { data: new Uint8Array(buf), filename: file.name, mode: selectedMode });
    } catch (e) {
      loadError = e.message;
    }
    // Reset so same file can be re-selected
    fileInput.value = '';
  }

  $: canLoad = state === 'ready' || state === 'halted' || state === 'error' || state === 'paused' || state === 'running';
</script>

<div class="rom-picker">
  <div class="section-label">ROMs</div>

  <div class="row">
    <select bind:value={selectedRom} disabled={!roms.length}>
      <option value={null}>— select a ROM —</option>
      {#each roms as rom}
        <option value={rom}>{rom.name}</option>
      {/each}
    </select>

    <select bind:value={selectedMode}>
      <option value="chip8">CHIP-8</option>
      <option value="schip">SCHIP</option>
    </select>

    <button on:click={loadSelected} disabled={!selectedRom || !canLoad}>
      Load
    </button>
  </div>

  <div class="row">
    <label class="upload-label">
      Upload custom ROM
      <input
        type="file"
        accept=".ch8,.c8,.rom"
        bind:this={fileInput}
        on:change={onFileChange}
        disabled={!canLoad}
        class="file-input"
      />
    </label>
  </div>

  {#if selectedRom}
    <div class="rom-info">
      {#if selectedRom.description}
        <p class="rom-desc">{selectedRom.description}</p>
      {/if}
      {#if selectedRom.instructions}
        <div class="rom-instructions">
          <span class="instr-label">Controls</span>
          {#each selectedRom.instructions.split('\n') as line}
            <p class="instr-line">{line}</p>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  {#if loadError}
    <p class="load-error">{loadError}</p>
  {/if}
</div>

<style>
  .rom-picker {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem;
    background: #12121e;
    border: 1px solid #2a2a3a;
    border-radius: 6px;
  }

  .section-label {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: #666;
  }

  .row {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    align-items: center;
  }

  select {
    flex: 1;
    min-width: 0;
    padding: 0.4rem 0.6rem;
    background: #1a1a2a;
    border: 1px solid #3a3a5a;
    border-radius: 4px;
    color: #c0c0e0;
    font-family: inherit;
    font-size: 0.8rem;
    cursor: pointer;
  }

  select:disabled { opacity: 0.5; cursor: not-allowed; }

  button {
    padding: 0.4rem 1rem;
    background: #1a2a4a;
    border: 1px solid #3a5a9a;
    border-radius: 4px;
    color: #a0c4ff;
    font-family: inherit;
    font-size: 0.8rem;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s;
  }

  button:hover:not(:disabled) { background: #1e3060; }
  button:disabled { opacity: 0.4; cursor: not-allowed; }

  .upload-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.75rem;
    background: #1a1a2a;
    border: 1px dashed #3a3a5a;
    border-radius: 4px;
    color: #888;
    font-size: 0.8rem;
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
  }

  .upload-label:hover {
    border-color: #6060a0;
    color: #c0c0e0;
  }

  .file-input {
    display: none;
  }

  .rom-info {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    padding: 0.5rem 0.6rem;
    background: #0e0e1a;
    border: 1px solid #252535;
    border-radius: 4px;
  }

  .rom-desc {
    font-size: 0.72rem;
    color: #777;
    font-style: italic;
  }

  .rom-instructions {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    margin-top: 0.2rem;
  }

  .instr-label {
    font-size: 0.6rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: #4a6a9a;
    margin-bottom: 0.1rem;
  }

  .instr-line {
    font-size: 0.75rem;
    color: #a0c0e0;
    font-family: monospace;
  }

  .load-error {
    font-size: 0.72rem;
    color: #f87171;
  }
</style>
