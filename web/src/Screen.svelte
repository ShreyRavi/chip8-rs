<script>
  import { afterUpdate } from 'svelte';

  export let pixels = new Uint8Array(0);
  export let displayWidth = 64;
  export let displayHeight = 32;

  let canvas;
  const PIXEL_ON  = '#a8ff60';
  const PIXEL_OFF = '#111820';

  afterUpdate(() => {
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const pw = canvas.width / displayWidth;
    const ph = canvas.height / displayHeight;

    for (let y = 0; y < displayHeight; y++) {
      for (let x = 0; x < displayWidth; x++) {
        ctx.fillStyle = pixels[y * displayWidth + x] ? PIXEL_ON : PIXEL_OFF;
        ctx.fillRect(x * pw, y * ph, pw, ph);
      }
    }
  });
</script>

<canvas
  bind:this={canvas}
  width={640}
  height={320}
  style="width: 100%; aspect-ratio: {displayWidth}/{displayHeight}; display: block; image-rendering: pixelated; background: {PIXEL_OFF}; border-radius: 4px; border: 1px solid #2a2a3a;"
></canvas>
