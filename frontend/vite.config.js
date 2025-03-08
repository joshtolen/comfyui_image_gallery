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
    outDir: 'dist',
    emptyOutDir: true,
    minify: 'esbuild',
    rollupOptions: {
      external: [],
      treeshake: false,
      output: {
        manualChunks: undefined,
        inlineDynamicImports: true,
        compact: true
      }
    },
    cssCodeSplit: false,
    assetsInlineLimit: 10000,
    chunkSizeWarningLimit: 500,
    sourcemap: false
  },
  optimizeDeps: {
    force: true,
    esbuildOptions: {
      target: 'es2020',
      supported: { bigint: true }
    }
  },
  esbuild: {
    jsxInject: `import React from 'react'`,
    target: 'es2020',
    supported: { 
      bigint: true 
    },
    // Keep comments
    legalComments: 'inline'
  }
});