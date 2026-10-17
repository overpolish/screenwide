// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { spawn, type ChildProcess } from "node:child_process";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import type { StorybookConfig } from "@storybook/react-vite";
import type { PluginOption } from "vite";

const NATIVE_PREVIEW_ENDPOINT = "/__screenwide/native-preview";
const STORY_ID_PATTERN = /^[a-z0-9-]+$/;
const scriptPath = fileURLToPath(
  new URL("../scripts/dev-storybook-native.mjs", import.meta.url),
);
const repositoryRoot = path.dirname(path.dirname(scriptPath));

// Without BROWSER, Storybook on macOS opens the URL in Chrome whenever any
// process named "Google Chrome" is running, including headless Chrome for
// Testing. "open" hands it to the system default browser instead. Other
// platforms already use the default browser and would treat "open" as an app.
if (process.platform === "darwin") process.env.BROWSER ??= "open";

let nativePreview: ChildProcess | undefined;

const nativePreviewPlugin: PluginOption = {
  configureServer(server) {
    server.middlewares.use((request, response, next) => {
      const url = new URL(request.url ?? "/", "http://localhost");
      if (
        request.method !== "POST" ||
        url.pathname !== NATIVE_PREVIEW_ENDPOINT
      ) {
        next();
        return;
      }

      const story = url.searchParams.get("story") ?? "";
      if (!STORY_ID_PATTERN.test(story)) {
        response.statusCode = 400;
        response.setHeader("content-type", "application/json");
        response.end(JSON.stringify({ error: "Invalid story ID" }));
        return;
      }

      if (nativePreview?.pid != null && !nativePreview.killed) {
        // The script forwards the signal to its own `tauri dev` process group.
        nativePreview.kill("SIGTERM");
      }

      const child = spawn(process.execPath, [scriptPath, "--story", story], {
        cwd: repositoryRoot,
        stdio: "inherit",
      });
      nativePreview = child;
      child.once("exit", () => {
        if (nativePreview === child) nativePreview = undefined;
      });

      response.statusCode = 202;
      response.setHeader("content-type", "application/json");
      response.end(JSON.stringify({ story }));
    });
  },
  name: "screenwide-native-preview",
};

const nativePreviewPreset = fileURLToPath(
  new URL("./native-preview-preset.ts", import.meta.url),
);

const config: StorybookConfig = {
  addons: [
    nativePreviewPreset,
    "@storybook/addon-docs",
    "@storybook/addon-themes",
  ],
  framework: {
    name: "@storybook/react-vite",
    options: {},
  },
  stories: ["../src/**/*.stories.@(js|jsx|mjs|ts|tsx)"],
  viteFinal: (viteConfig) => {
    viteConfig.server ??= {};
    viteConfig.server.watch = {
      ...viteConfig.server.watch,
      ignored: ["**/dist/**", "**/src-tauri/**", "**/storybook-static/**"],
    };
    viteConfig.plugins ??= [];
    viteConfig.plugins.push(nativePreviewPlugin);

    return viteConfig;
  },
};

export default config;
