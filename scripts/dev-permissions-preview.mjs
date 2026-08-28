// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { spawnSync } from "node:child_process";
import { argv, env, exit, platform } from "node:process";

const command = platform === "win32" ? "pnpm.cmd" : "pnpm";
const arguments_ = argv.slice(2);
if (arguments_[0] === "--") arguments_.shift();
const result = spawnSync(command, ["tauri", "dev", ...arguments_], {
  env: {
    ...env,
    SCREENWIDE_SHOW_PERMISSIONS: "1",
    VITE_SCREENWIDE_PERMISSIONS_PREVIEW: "1",
  },
  stdio: "inherit",
});

if (result.error) throw result.error;
exit(result.status ?? 1);
