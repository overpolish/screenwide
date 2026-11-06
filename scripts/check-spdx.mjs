// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { extname } from "node:path";
import process from "node:process";

const COPYRIGHT = "SPDX-FileCopyrightText: 2026 overpolish";
const GPL = "GPL-3.0-or-later";
const ELASTIC = "Elastic-2.0";

// The paid tools (Ruler, References, Glide) are under the Elastic License 2.0
// (`LICENSES/Elastic-2.0.txt`); everything else is under the GPL. A path that
// ends in "/" covers the folder below it.
const ELASTIC_PATHS = [
  "src/features/glide/",
  "src/features/reference/",
  "src/features/ruler/",
  "src-tauri/crates/licensing/",
  "src-tauri/crates/osc/src/gpu/shaders/reference.wgsl",
  "src-tauri/crates/osc/src/gpu/vertices/ruler.rs",
  "src-tauri/src/app_windows/glide_island_control.rs",
  "src-tauri/src/app_windows/platform/glide_island.rs",
  "src-tauri/src/app_windows/screenshot_region/region_osc/input/ruler_input.rs",
  "src-tauri/src/app_windows/screenshot_region/region_osc/ruler/",
  "src-tauri/src/app_windows/screenshot_region/region_osc/ruler.rs",
  "src-tauri/src/app_windows/screenshot_region/region_osc/state/ruler_updates.rs",
  "src-tauri/src/glide/",
  "src-tauri/src/glide.rs",
  "src-tauri/src/references/",
  "src-tauri/src/references.rs",
  "src-tauri/src/ruler/",
  "src-tauri/src/ruler.rs",
  "workers/license/",
];

const CHECKED_EXTENSIONS = new Set([
  ".c",
  ".css",
  ".h",
  ".html",
  ".ftl",
  ".js",
  ".m",
  ".md",
  ".metal",
  ".mjs",
  ".plist",
  ".rs",
  ".svg",
  ".toml",
  ".ts",
  ".tsx",
  ".wgsl",
  ".yaml",
  ".yml",
]);
// A licence text is the terms themselves; it carries no other licence's header.
const EXCLUDED_FILES = new Set(["pnpm-lock.yaml", "TOOLS-LICENSE.md"]);
// ts-rs writes a fixed generated-file note as the first line of each binding.
const EXCLUDED_DIRECTORIES = ["src-tauri/vendor/", "src/bindings/"];

const licenseOf = (file) =>
  ELASTIC_PATHS.some((path) =>
    path.endsWith("/") ? file.startsWith(path) : file === path,
  )
    ? ELASTIC
    : GPL;

const trackedFiles = execFileSync(
  "git",
  ["ls-files", "--cached", "--others", "--exclude-standard", "-z"],
  { encoding: "utf8" },
)
  .split("\0")
  .filter(Boolean);

const problems = [];
for (const file of trackedFiles) {
  if (
    !existsSync(file) ||
    EXCLUDED_FILES.has(file) ||
    EXCLUDED_DIRECTORIES.some((directory) => file.startsWith(directory)) ||
    !CHECKED_EXTENSIONS.has(extname(file))
  ) {
    continue;
  }
  const header = readFileSync(file, "utf8").split(/\r?\n/, 12).join("\n");
  const expected = licenseOf(file);
  if (!header.includes(COPYRIGHT)) {
    problems.push(`${file}: no SPDX copyright line`);
  } else if (!header.includes(`SPDX-License-Identifier: ${expected}`)) {
    problems.push(`${file}: should be SPDX-License-Identifier: ${expected}`);
  }
}

if (problems.length > 0) {
  console.error("These tracked files have a missing or wrong SPDX header:");
  for (const problem of problems) console.error(`- ${problem}`);
  process.exitCode = 1;
}
