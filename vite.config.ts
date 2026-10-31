import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    host: '0.0.0.0',
    port: 3000,
    proxy: {
      '/api/duckdns': {
        target: 'https://prysmview.duckdns.org',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/duckdns/, ''),
      },
    },
  },
});
