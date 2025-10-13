import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    port: 1420,
    strictPort: true,
    host: true,
    hmr: {
      port: 1421,
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
  },
  define: {
    global: 'globalThis',
  },
  resolve: {
    alias: {
      'web-worker': path.resolve(__dirname, 'src/utils/dummy-web-worker.js'),
      '@tauri-apps/api/core': path.resolve(__dirname, 'src/utils/tauri-api-mock.js'),
    },
  },
})