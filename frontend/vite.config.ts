import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

const isTauri =
  Boolean(process.env.TAURI_CONFIG_DIR) ||
  process.env.TAURI === 'true' ||
  Boolean(process.env.TAURI_PLATFORM) ||
  Boolean(process.env.TAURI_ENV_PLATFORM)

const alias: Record<string, string> = {
  'web-worker': path.resolve(__dirname, 'src/utils/dummy-web-worker.js'),
}

if (!isTauri) {
  alias['@tauri-apps/api/core'] = path.resolve(__dirname, 'src/utils/tauri-api-mock.js')
}

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    port: 1420,
    strictPort: true,
    host: '127.0.0.1',
    hmr: {
      host: '127.0.0.1',
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
    alias,
  },
})
