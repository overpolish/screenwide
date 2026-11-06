// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// The public website, built from the app's own components: a page of its own
// in `src/site`, served and built apart from the app's windows.
export default defineConfig({
  build: {
    emptyOutDir: true,
    outDir: "../../dist-site",
  },
  plugins: [react(), tailwindcss()],
  root: "src/site",
  server: { port: 4173 },
});
