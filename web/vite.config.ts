import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

// In dev, run `sitroom serve --dev-origin http://localhost:5173` and open the printed
// http://localhost:5173/?boot=<token> link. The proxy forwards API and WebSocket traffic.
export default defineConfig({
  plugins: [vue()],
  build: {
    rolldownOptions: {
      // Optional peers of @osdlc/markdown-render (MarkdownText.vue), which
      // pnpm installs anyway. Math is off, and a mermaid fence shows its
      // source, so neither is bundled.
      external: ['katex', 'mermaid'],
    },
  },
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      '/api': { target: 'http://127.0.0.1:7878', ws: true },
    },
  },
})
