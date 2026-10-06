// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Runs the app with recording and replay failures faked, so each alert and
// input warning can be seen without the hardware that causes it. The first
// argument names the faults (see src-tauri/src/fault.rs), comma separated:
//
//   pnpm dev:recording-failures camera-mode,stop
//
// Anything after it goes to `tauri dev`.

import { argv, env, exit } from "node:process";

import { spawnPnpmSync } from "./pnpm.mjs";

const arguments_ = argv.slice(2);
if (arguments_[0] === "--") arguments_.shift();
// PowerShell reads `camera-mode,stop` as an array, and pnpm's PowerShell shim
// passes it on joined with spaces, so spaces separate faults as well.
const faults =
  arguments_[0] && !arguments_[0].startsWith("-")
    ? arguments_
        .shift()
        .split(/[\s,]+/)
        .filter(Boolean)
        .join(",")
    : "camera-mode";
const result = spawnPnpmSync(["tauri", "dev", ...arguments_], {
  env: {
    ...env,
    SCREENWIDE_FAULT: faults,
    SCREENWIDE_SHOW_RECORDING_BAR: "1",
  },
  stdio: "inherit",
});

if (result.error) throw result.error;
exit(result.status ?? 1);
