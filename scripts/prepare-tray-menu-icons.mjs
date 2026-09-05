// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { copyFile, mkdir, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

import { Resvg } from "@resvg/resvg-js";
import {
  ClipboardPaste,
  DoorOpen,
  Monitor,
  Pause,
  Play,
  Ruler,
  ScanText,
  Settings,
  Square,
  Trash2,
  X,
} from "lucide-react";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

const output = new URL("../src-tauri/icons/menu/", import.meta.url);
await mkdir(output, { recursive: true });

// Keep the native menu gutter, with a 14-point glyph inside its 18-point slot
// (rendered at 2×). Windows gets the same proportional reduction. Partial
// opacity softens the native tint to neutral while retaining state contrast.
// Commit the PNGs so normal builds need neither Node nor an SVG renderer.
for (const [name, icon] of Object.entries({
  cancel: X,
  clipboard: ClipboardPaste,
  discard: Trash2,
  open: Monitor,
  pause: Pause,
  quit: DoorOpen,
  resume: Play,
  ruler: Ruler,
  settings: Settings,
  stop: Square,
  text: ScanText,
})) {
  const glyph = renderToStaticMarkup(
    createElement(icon, {
      color: "black",
      size: 28,
      strokeWidth: 2,
      x: 4,
      y: 4,
    }),
  );
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="36" height="36"><g opacity="0.72">${glyph}</g></svg>`;
  await writeFile(
    new URL(`${name}.png`, output),
    new Resvg(svg).render().asPng(),
  );
}

await copyFile(
  fileURLToPath(
    new URL("../node_modules/lucide-react/LICENSE", import.meta.url),
  ),
  new URL("LICENSE", output),
);
