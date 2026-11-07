// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Scales the app icon into the unplated sizes of the Store package's small
// logo, which the taskbar and Start draw it at
// (`scripts/package-msix.mjs` indexes them). `pnpm icons:prepare` runs it
// after rendering the icon; commit the PNGs, so packaging needs no renderer.

import { mkdir, readFile, rm, writeFile } from "node:fs/promises";

import { Resvg } from "@resvg/resvg-js";

const source = new URL(
  "../src-tauri/icons/ScreenwideLogo.png",
  import.meta.url,
);
const output = new URL("../src-tauri/msix/Assets/", import.meta.url);
const sizes = [16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, 256];

function scale(png, size) {
  const href = `data:image/png;base64,${png.toString("base64")}`;
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}"><image href="${href}" width="${size}" height="${size}" image-rendering="optimizeQuality"/></svg>`;
  return new Resvg(svg).render().asPng();
}

const original = await readFile(source);
const sourceSize = original.readUInt32BE(16);
await rm(output, { force: true, recursive: true });
await mkdir(output, { recursive: true });
for (const size of sizes) {
  // resvg's filter does not prefilter, so each pass at most halves the
  // picture; the last one makes up the rest.
  let png = original;
  for (let step = sourceSize / 2; step >= size; step /= 2) {
    png = scale(png, step);
  }
  await writeFile(
    new URL(`Square44x44Logo.targetsize-${size}_altform-unplated.png`, output),
    scale(png, size),
  );
}
