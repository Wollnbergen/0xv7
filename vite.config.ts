// vite.config.ts - Wallet build config (SDK/WASM compat)
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  root: path.resolve(__dirname, 'sl/sultan-interop/telegram-mini-app'),
  build: {
    rollupOptions: {
      input: path.resolve(__dirname, 'public/index.html'),
    },
    outDir: 'dist',
    minify: 'esbuild',
  },
  server: {
    port: 5000,
    host: '0.0.0.0',
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'src'),
    },
  },
  // Fix temp timestamp (Vite 7+)
  cacheDir: '.vite-temp',
});