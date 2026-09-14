/// <reference types="vitest/config" />
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Tauri reads this when it drives the dev server / build. Unset for a plain
// web `npm run build`, so the web bundle is unaffected by the desktop/Android
// packaging.
const tauriPlatform = process.env.TAURI_ENV_PLATFORM;
const tauriDebug = !!process.env.TAURI_ENV_DEBUG;
const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],

  // Don't let Vite wipe the screen over Rust compiler errors during `tauri dev`.
  clearScreen: false,

  server: {
    // Tauri expects a fixed port; fail rather than silently hop to another.
    port: 5173,
    strictPort: true,
    // Bind to the LAN host only when Tauri sets it (Android device dev);
    // localhost-only otherwise.
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 5174 } : undefined,
    watch: {
      // The Rust side has its own watcher; don't churn on target/ output.
      ignored: ['**/src-tauri/**'],
    },
  },

  // Expose TAURI_ENV_* to the frontend alongside the usual VITE_*.
  envPrefix: ['VITE_', 'TAURI_ENV_*'],

  build: tauriPlatform
    ? {
        // Match the floor of each platform's webview. WebKitGTK (Linux) and
        // recent Android WebView are modern; safari13 is the safe syntax floor.
        target: tauriPlatform === 'windows' ? 'chrome105' : 'safari13',
        minify: tauriDebug ? false : 'esbuild',
        sourcemap: tauriDebug,
      }
    : {},

  test: {
    // Default to node: the suite is pure DSP math with no DOM. Component
    // tests opt into jsdom per-file via `// @vitest-environment jsdom`.
    environment: 'node',
    globals: true,
  },
});
