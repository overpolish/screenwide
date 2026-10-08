// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Regenerates the ts-rs bindings and fails if the result differs from what is
// in the index. The bindings are deleted first so a type that no longer exists
// leaves no stale file behind: ts-rs only writes, it never removes.

import { execFileSync } from "node:child_process";
import { rmSync } from "node:fs";
import process from "node:process";

const BINDINGS_DIR = "src/bindings";

rmSync(BINDINGS_DIR, { recursive: true, force: true });

execFileSync("cargo", [
  "test",
  "--manifest-path",
  "src-tauri/Cargo.toml",
  "export_bindings",
]);

// Porcelain lines are `XY path`, X the index and Y the working tree. Only Y
// matters: bindings that are staged but not yet committed are current.
const changed = execFileSync(
  "git",
  ["status", "--porcelain", "--", BINDINGS_DIR],
  { encoding: "utf8" },
)
  .split("\n")
  .filter((line) => line.length > 3 && line[1] !== " ")
  .join("\n");

if (changed !== "") {
  console.error(
    `Out-of-date ts-rs bindings in ${BINDINGS_DIR}. ` +
      "Regenerate them and stage the result.",
  );
  console.error(changed);
  process.exitCode = 1;
}
