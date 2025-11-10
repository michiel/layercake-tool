import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

const isTauri =
  Boolean(process.env.TAURI_CONFIG_DIR) ||
  process.env.TAURI === 'true' ||
  Boolean(process.env.TAURI_PLATFORM) ||
  Boolean(process.env.TAURI_ENV_PLATFORM)

const alias: Record<string, string> = {
  '@': path.resolve(__dirname, './src'),
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
    host: true,
    hmr: {
      port: 1421,
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: {
          // Vendor chunks
          'vendor-react': ['react', 'react-dom', 'react-router-dom'],
          'vendor-ui': [
            '@radix-ui/react-accordion',
            '@radix-ui/react-avatar',
            '@radix-ui/react-checkbox',
            '@radix-ui/react-dialog',
            '@radix-ui/react-dropdown-menu',
            '@radix-ui/react-hover-card',
            '@radix-ui/react-label',
            '@radix-ui/react-popover',
            '@radix-ui/react-progress',
            '@radix-ui/react-scroll-area',
            '@radix-ui/react-select',
            '@radix-ui/react-separator',
            '@radix-ui/react-slider',
            '@radix-ui/react-slot',
            '@radix-ui/react-switch',
            '@radix-ui/react-tabs',
            '@radix-ui/react-tooltip',
          ],
          'vendor-apollo': ['@apollo/client', 'graphql', 'graphql-ws'],
          'vendor-reactflow': ['reactflow'],
          'vendor-assistant': [
            '@assistant-ui/react',
            '@assistant-ui/react-markdown',
          ],
          // Layout engine - now much smaller with dagre instead of ELK
          'layout': ['dagre'],
          // Heavy visualization libraries (lazy loaded)
          // These will be code-split automatically via dynamic imports
        },
      },
    },
    chunkSizeWarningLimit: 600,
  },
  define: {
    global: 'globalThis',
  },
  resolve: {
    alias,
  },
})
