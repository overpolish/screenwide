// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { resolve } from "node:path";

import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// The public website, built from the app's own components: a page of its own
// in `src/site`, served and built apart from the app's windows. Each page has
// its own file, so the host serves /privacy as privacy.html.
export default defineConfig({
  build: {
    emptyOutDir: true,
    outDir: "../../dist-site",
    rolldownOptions: {
      input: {
        index: resolve(import.meta.dirname, "src/site/index.html"),
        privacy: resolve(import.meta.dirname, "src/site/privacy.html"),
      },
    },
  },
  plugins: [react(), tailwindcss()],
  root: "src/site",
  server: { port: 4173 },
});
