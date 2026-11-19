// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { existsSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { arch as hostArch, env } from "node:process";

/** The newest Windows SDK's copy of `name` (makeappx, makepri), for the
 * machine this runs on, unless the variable of its name in capitals names
 * one. */
export function findSdkTool(name) {
  const override = env[name.toUpperCase()];
  if (override) return override;
  const kits = "C:\\Program Files (x86)\\Windows Kits\\10\\bin";
  const host = hostArch === "arm64" ? "arm64" : "x64";
  const versions = existsSync(kits)
    ? readdirSync(kits)
        .filter((entry) => /^10\.\d+\.\d+\.\d+$/.test(entry))
        .sort((a, b) => a.localeCompare(b, undefined, { numeric: true }))
        .reverse()
    : [];
  for (const sdk of versions) {
    for (const folder of [host, "x64"]) {
      const tool = join(kits, sdk, folder, `${name}.exe`);
      if (existsSync(tool)) return tool;
    }
  }
  throw new Error(`${name} was not found; install the Windows SDK`);
}
