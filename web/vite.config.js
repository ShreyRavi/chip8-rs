import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

export default defineConfig({
  base: process.env.VITE_BASE ?? '/',
  plugins: [
    wasm(),
    topLevelAwait(),
    svelte(),
  ],
  optimizeDeps: {
    exclude: ['chip8-wasm'],
  },
});
