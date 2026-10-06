// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { argv, env, exit } from "node:process";

import { spawnPnpmSync } from "./pnpm.mjs";

const arguments_ = argv.slice(2);
if (arguments_[0] === "--") arguments_.shift();
const result = spawnPnpmSync(["tauri", "dev", ...arguments_], {
  env: {
    ...env,
    SCREENWIDE_SHOW_PERMISSIONS: "1",
    VITE_SCREENWIDE_PERMISSIONS_PREVIEW: "1",
  },
  stdio: "inherit",
});

if (result.error) throw result.error;
exit(result.status ?? 1);
