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

/** The private side of Vite's bundled dev mode that `awaitCurrentBundle` waits on. */
type BundledDev = {
  devEngine: {
    ensureLatestBuildOutput(): Promise<void>;
    getBundleState(): Promise<{
      hasStaleOutput: boolean;
      lastBuildErrored: boolean;
    }>;
  };
  waitForInitialBuildFinish(): Promise<void>;
};

/**
 * Holds requests until bundled dev mode has a current bundle to serve. Vite
 * answers a page requested while the bundle is missing, before the first one,
 * or stale, after an edit it applied as a hot update, with a placeholder that
 * leaves only on one reload broadcast, sent when the bundle is ready, so a
 * window whose socket connects after that broadcast keeps the placeholder for
 * good. WebView2 brings the app's many windows up one after another, slowly
 * enough that several land in that gap on Windows, at launch and again when
 * a Rust change relaunches the app against the running server; waiting here
 * serves them the app. Both waits settle when a build fails, so Vite's error
 * overlay still shows.
 */
const awaitCurrentBundle = (): Plugin => ({
  configureServer(server) {
    const bundledDev = server.environments.client.bundledDev as unknown as
      BundledDev | undefined;
    if (!bundledDev) return;
    let firstBundle: Promise<void> | undefined;
    let latestBundle: Promise<void> | undefined;
    const currentBundle = async (isPage: boolean) => {
      firstBundle ??= bundledDev.waitForInitialBuildFinish();
      await firstBundle;
      // Only a page load is answered with the placeholder; a script or asset
      // asked for by a page already running takes what the bundle holds.
      if (!isPage) return;
      const state = await bundledDev.devEngine.getBundleState();
      if (!state.hasStaleOutput || state.lastBuildErrored) return;
      latestBundle ??= bundledDev.devEngine
        .ensureLatestBuildOutput()
        .finally(() => {
          latestBundle = undefined;
        });
      await latestBundle;
    };
    server.middlewares.use((req, _res, next) => {
      currentBundle(req.headers["sec-fetch-dest"] === "document").then(
        () => {
          next();
        },
        () => {
          next();
        },
      );
    });
  },
  name: "screenwide:await-current-bundle",
});

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [react(), tailwind(), awaitCurrentBundle()],

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
