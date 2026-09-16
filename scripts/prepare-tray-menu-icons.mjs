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
  PenTool,
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
const windowsOutput = new URL(
  "../src-tauri/icons/menu/windows/",
  import.meta.url,
);
await mkdir(output, { recursive: true });
await mkdir(windowsOutput, { recursive: true });

// macOS renders menu images at 2×.
// The Windows adapter resizes this 64px source into DPI-sized native bitmaps,
// bypassing muda's fixed 16px conversion.
// Commit the PNGs so normal builds need neither Node nor an SVG renderer.
for (const [name, icon] of Object.entries({
  annotate: PenTool,
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
  // A full-alpha mask: as a template image, AppKit paints it in the menu's
  // label colour, so the glyph matches the item text exactly.
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="36" height="36">${glyph}</svg>`;
  await writeFile(
    new URL(`${name}.png`, output),
    new Resvg(svg).render().asPng(),
  );

  const windowsGlyph = renderToStaticMarkup(
    createElement(icon, {
      color: "black",
      size: 56,
      strokeWidth: 2,
      x: 4,
      y: 4,
    }),
  );
  const windowsSvg = `<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64">${windowsGlyph}</svg>`;
  await writeFile(
    new URL(`${name}.png`, windowsOutput),
    new Resvg(windowsSvg).render().asPng(),
  );
}

await copyFile(
  fileURLToPath(
    new URL("../node_modules/lucide-react/LICENSE", import.meta.url),
  ),
  new URL("LICENSE", output),
);
