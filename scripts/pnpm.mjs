// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { spawn, spawnSync } from "node:child_process";
import { env, execPath, platform } from "node:process";

// Node refuses to spawn the `pnpm.cmd` shim without a shell
// (CVE-2024-27980), so pnpm is reached through the entry point it reports
// when it runs a script: a standalone executable or a script for node. Only
// a bare `node scripts/...` invocation falls back to the shim, via the shell.
const pnpm = (() => {
  const entry = env.npm_execpath;
  if (entry == null) {
    return { command: "pnpm", prefix: [], shell: platform === "win32" };
  }
  if (/\.[cm]?js$/i.test(entry)) {
    return { command: execPath, prefix: [entry], shell: false };
  }
  return { command: entry, prefix: [], shell: false };
})();

export const spawnPnpm = (pnpmArguments, options) =>
  spawn(pnpm.command, [...pnpm.prefix, ...pnpmArguments], {
    ...options,
    shell: pnpm.shell,
  });

export const spawnPnpmSync = (pnpmArguments, options) =>
  spawnSync(pnpm.command, [...pnpm.prefix, ...pnpmArguments], {
    ...options,
    shell: pnpm.shell,
  });
