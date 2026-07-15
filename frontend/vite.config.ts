import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

const alias: Record<string, string> = {
  '@': path.resolve(__dirname, './src'),
  'web-worker': path.resolve(__dirname, 'src/utils/dummy-web-worker.js'),
}

// In dev, the Vite server proxies API traffic to the Rust server so the app can
// use same-origin relative endpoints (matching the embedded production build).
const apiTarget = process.env.VITE_API_BASE_URL || 'http://localhost:3001'

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
    proxy: {
      '/api': { target: apiTarget, changeOrigin: true },
      '/graphql': { target: apiTarget, changeOrigin: true, ws: true },
      '/projections': { target: apiTarget, changeOrigin: true, ws: true },
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
