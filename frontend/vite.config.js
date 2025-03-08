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
    minify: 'esbuild',
    // Use esbuild only, avoid rollup completely
    rollupOptions: {
      external: [],
      treeshake: false,
      output: {
        manualChunks: undefined,
        inlineDynamicImports: true,
        compact: true
      }
    },
    // Try to generate a single bundle file
    cssCodeSplit: false,
    assetsInlineLimit: 100000000,
    chunkSizeWarningLimit: 100000000,
    sourcemap: false,
    manifest: false,
    write: true
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