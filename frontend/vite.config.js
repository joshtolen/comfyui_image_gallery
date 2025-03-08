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
    rollupOptions: {
      // Disable features that might cause issues
      treeshake: false,
      output: {
        manualChunks: undefined
      }
    }
  },
  optimizeDeps: {
    // Force inclusion of dependencies that might be problematic
    include: ['react', 'react-dom', 'react-router-dom']
  }
});