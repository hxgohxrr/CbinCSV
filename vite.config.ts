import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: { '@lang': path.resolve(__dirname, 'src/lang') }
  },
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: { target: 'chrome105', minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false }
})
