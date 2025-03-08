import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:9999',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, '')
      },
      '/static': {
        target: 'http://localhost:9999',
        changeOrigin: true,
      }
    }
  },
  build: {
    outDir: '../static/react',
    emptyOutDir: true,
  }
});