// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig, type Plugin, type ViteDevServer } from "vite";

const host = process.env.TAURI_DEV_HOST;

/**
 * Tailwind's plugins, with the hook that refreshes its CSS when an asset
 * changes skipped where Vite runs it without a dev server, as bundled dev
 * mode does: the hook reaches for the server and throws. Bundled mode
 * rebuilds the stylesheet from the files Tailwind scanned all the same.
 */
const tailwind = (): Plugin[] =>
  tailwindcss().map((plugin) => {
    const hotUpdate = plugin.hotUpdate;
    if (typeof hotUpdate !== "function") return plugin;
    return {
      ...plugin,
      hotUpdate(context) {
        const server = context.server as ViteDevServer | undefined;
        return server ? hotUpdate.call(this, context) : undefined;
      },
    };
  });

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [react(), tailwind()],

  // Stamps what the windows keep in storage with this build, so a window
  // launched with it never takes up what an earlier build left there.
  define: {
    "import.meta.env.SCREENWIDE_BUILD": JSON.stringify(Date.now().toString(36)),
  },

  // Every window is served from the same local WebView bundle. The desktop app
  // does not pay a network-transfer penalty for keeping that shared startup
  // path together, so use a limit that reflects the packaged application.
  build: {
    chunkSizeWarningLimit: 1024,
  },
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // Every window loads at launch, all at once and from one dev server. Served
  // a module at a time, they queue thousands of requests on the few
  // connections the WebView keeps to it and take seconds to come up; served
  // bundled, as a packaged app's are, they load as quickly as in release.
  // Only for the app, which the Tauri CLI marks: Storybook shares this config
  // and gains nothing from an experimental mode.
  experimental: {
    bundledDev: process.env.TAURI_ENV_PLATFORM !== undefined,
  },
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    hmr: host
      ? {
          host,
          port: 1421,
          protocol: "ws",
        }
      : undefined,
    host: host || false,
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
