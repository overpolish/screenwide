// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Renders the app icon for everywhere but macOS from the Icon Composer
// document, edge to edge. macOS draws every app's icon on Apple's grid,
// whose artwork sits inside a margin a tenth of each side wide; Windows and
// browsers draw the file as it is, so an icon carrying that margin shows a
// fifth smaller than every other app's. The macOS icon (`icon.icns`,
// `Assets.car`) is left as it is. Run on a Mac with Xcode 26 after the icon
// changes and commit what it writes, so building needs no renderer.

import { spawnSync } from "node:child_process";
import { copyFile, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { exit, platform } from "node:process";
import { fileURLToPath } from "node:url";

import { Resvg } from "@resvg/resvg-js";

if (platform !== "darwin") {
  console.error("The app icons are rendered with Icon Composer, on macOS");
  exit(1);
}

const workspace = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const icons = join(workspace, "src-tauri/icons");
const document = join(icons, "Screenwide.icon");

// What `tauri icon` writes that is drawn outside macOS; its `icon.icns` and
// its mobile icons are not used.
const generated = [
  "32x32.png",
  "64x64.png",
  "128x128.png",
  "128x128@2x.png",
  "icon.ico",
  "icon.png",
  "Square107x107Logo.png",
  "Square142x142Logo.png",
  "Square150x150Logo.png",
  "Square284x284Logo.png",
  "Square30x30Logo.png",
  "Square310x310Logo.png",
  "Square44x44Logo.png",
  "Square71x71Logo.png",
  "Square89x89Logo.png",
  "StoreLogo.png",
];

function run(command, args) {
  const result = spawnSync(command, args, { cwd: workspace, stdio: "inherit" });
  if (result.status !== 0)
    throw new Error(`${command} exited with status ${String(result.status)}`);
}

// Icon Composer ships inside Xcode 26, with the command-line renderer
// beside it; the `ictool` on Xcode's own path cannot export pictures.
function ictool() {
  const developer = spawnSync("xcode-select", ["-p"], { encoding: "utf8" });
  if (developer.status !== 0) throw new Error("Xcode is not installed");
  return resolve(
    developer.stdout.trim(),
    "../Applications/Icon Composer.app/Contents/Executables/ictool",
  );
}

/** The icon `size` px square, as Icon Composer draws it, edge to edge.
 * Icon Composer writes 16 bits a channel; it is drawn again at 8, which is
 * all an icon file holds and a quarter of the bytes. */
async function rendered(directory, size) {
  const raw = join(directory, `icon-${String(size)}.png`);
  run(ictool(), [
    document,
    "--export-image",
    "--output-file",
    raw,
    "--platform",
    "macOS",
    "--rendition",
    "Default",
    "--width",
    String(size),
    "--height",
    String(size),
    "--scale",
    "1",
  ]);
  const href = `data:image/png;base64,${(await readFile(raw)).toString("base64")}`;
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${String(size)}" height="${String(size)}"><image href="${href}" width="${String(size)}" height="${String(size)}"/></svg>`;
  return new Resvg(svg).render().asPng();
}

const scratch = await mkdtemp(join(tmpdir(), "screenwide-icons-"));
try {
  const logo = join(icons, "ScreenwideLogo.png");
  await writeFile(logo, await rendered(scratch, 1024));
  // A tab draws it at 16 or 32 px, so a larger file only costs bytes.
  await writeFile(
    join(workspace, "src/site/public/favicon.png"),
    await rendered(scratch, 256),
  );
  const out = join(scratch, "tauri");
  run("pnpm", ["exec", "tauri", "icon", logo, "--output", out]);
  for (const name of generated)
    await copyFile(join(out, name), join(icons, name));
  run("node", ["scripts/prepare-msix-logos.mjs"]);
} finally {
  await rm(scratch, { force: true, recursive: true });
}
