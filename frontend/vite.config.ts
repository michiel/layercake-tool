import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

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
      'web-worker': './src/utils/dummy-web-worker.js',
      '@tauri-apps/api/core': './src/utils/tauri-api-mock.js',
    },
  },
})