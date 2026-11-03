// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Regenerates the ts-rs bindings and fails if the result differs from what is
// in the index. They are generated into a scratch directory and only then
// swapped in, so a type that no longer exists leaves no stale file behind
// (ts-rs only writes, it never removes) and a build that fails leaves the
// bindings as they were. Cargo's `[env]` does not override a variable already
// set, so `TS_RS_EXPORT_DIR` here wins over `.cargo/config.toml`.

import { execFileSync } from "node:child_process";
import { cpSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import process from "node:process";

const BINDINGS_DIR = "src/bindings";

const scratch = mkdtempSync(path.join(tmpdir(), "screenwide-bindings-"));

try {
  execFileSync(
    "cargo",
    [
      "test",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "--workspace",
      "export_bindings",
    ],
    {
      env: { ...process.env, TS_RS_EXPORT_DIR: scratch },
      stdio: ["ignore", "ignore", "inherit"],
    },
  );
} catch {
  rmSync(scratch, { force: true, recursive: true });
  console.error("Generating the ts-rs bindings failed; they are unchanged.");
  process.exit(1);
}

rmSync(BINDINGS_DIR, { force: true, recursive: true });
cpSync(scratch, BINDINGS_DIR, { recursive: true });
rmSync(scratch, { force: true, recursive: true });

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
