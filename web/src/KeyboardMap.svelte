<script>
  export let pressedKeys = new Set();

  // Layout: physical key → chip8 key
  const ROWS = [
    [{ k: '1', c: 0x1 }, { k: '2', c: 0x2 }, { k: '3', c: 0x3 }, { k: '4', c: 0xC }],
    [{ k: 'Q', c: 0x4 }, { k: 'W', c: 0x5 }, { k: 'E', c: 0x6 }, { k: 'R', c: 0xD }],
    [{ k: 'A', c: 0x7 }, { k: 'S', c: 0x8 }, { k: 'D', c: 0x9 }, { k: 'F', c: 0xE }],
    [{ k: 'Z', c: 0xA }, { k: 'X', c: 0x0 }, { k: 'C', c: 0xB }, { k: 'V', c: 0xF }],
  ];
</script>

<div class="keymap">
  <div class="section-label">Keyboard → CHIP-8</div>
  <div class="grid">
    {#each ROWS as row}
      <div class="row">
        {#each row as key}
          <div class="key" class:active={pressedKeys.has(key.c)}>
            <span class="phys">{key.k}</span>
            <span class="chip">{key.c.toString(16).toUpperCase()}</span>
          </div>
        {/each}
      </div>
    {/each}
  </div>
  <p class="hint">Space to pause/resume</p>
</div>

<style>
  .keymap {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
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

  .grid {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .row {
    display: flex;
    gap: 0.25rem;
  }

  .key {
    width: 44px;
    height: 44px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: #1a1a2a;
    border: 1px solid #2a2a3a;
    border-radius: 4px;
    transition: background 0.1s, border-color 0.1s;
    gap: 2px;
  }

  .key.active {
    background: #1a3060;
    border-color: #4080c0;
  }

  .phys {
    font-size: 0.65rem;
    font-weight: 700;
    color: #c0c0e0;
  }

  .chip {
    font-size: 0.55rem;
    color: #6080a0;
  }

  .hint {
    font-size: 0.65rem;
    color: #555;
    margin-top: 0.1rem;
  }
</style>
