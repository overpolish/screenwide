// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { mkdir, readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

import { Resvg } from "@resvg/resvg-js";
import { History } from "lucide-react";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

// The tray shows Delayed Screenshot's seconds left as one of these, from the
// longest delay Settings offers down to one, and a running replay buffer as
// the History icon its menu item wears. Like the recording, paused and
// loading icons, each is the tray mark's arc with its own glyph where the dot
// sits. They are full-alpha masks: macOS shows them as template images and
// Windows recolours them to the taskbar's foreground. Only their alpha
// counts, but the glyphs are white like the arc they sit on, so every tray
// icon looks the same as a file too.
// Commit the PNGs so normal builds need neither Node nor an SVG renderer.
const LONGEST_DELAY = 10;
const SIZE = 32;
const INK = "white";

/** The arc's top edge: everything above it in the loading icon is its dots. */
const ARC_TOP = 12;
/** Where the state icons centre their glyph, and the box it fills: as tall as
 * the pause bars, as wide as the loading dots. */
const GLYPH_CENTRE = [18.5, 7];
const GLYPH_HEIGHT = 12;
const GLYPH_WIDTH = 16;
/** The History icon's size and line weight. Its own weight would be under a
 * pixel at this size and blur; this one keeps its hands legible. */
const HISTORY_SIZE = 13;
const HISTORY_STROKE = 1.75;

const icons = new URL("../src-tauri/icons/", import.meta.url);
const output = new URL("tray-countdown/", icons);
const font = fileURLToPath(
  new URL(
    "../src-tauri/assets/RobotoMono-VariableFont_wght.ttf",
    import.meta.url,
  ),
);
await mkdir(output, { recursive: true });

const arc = (await readFile(new URL("tray-loading.png", icons))).toString(
  "base64",
);
const fontOptions = {
  defaultFontFamily: "Roboto Mono",
  fontFiles: [font],
  loadSystemFonts: false,
};
// Rendered this many times larger to measure a number's ink to a fraction of
// a pixel.
const MEASURE_ZOOM = 16;

/** How the ink `content` draws lies, in icon pixels: its bounds, and the
 * horizontal centre of the ink in its lower half. Measured on a padded
 * canvas, so ink past the icon's edge is still counted. */
function measureInk(content) {
  const pad = SIZE;
  const canvas = SIZE + pad * 2;
  const image = new Resvg(
    `<svg xmlns="http://www.w3.org/2000/svg" width="${canvas}" height="${canvas}"><g transform="translate(${pad} ${pad})">${content}</g></svg>`,
    { fitTo: { mode: "zoom", value: MEASURE_ZOOM }, font: fontOptions },
  ).render();
  // `pixels` is a native getter that builds a new buffer on every read.
  const { height, pixels, width } = image;
  const alphaAt = (x, y) => pixels[(y * width + x) * 4 + 3];
  let [left, top, right, bottom] = [Infinity, Infinity, -Infinity, -Infinity];
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      if (alphaAt(x, y) === 0) continue;
      left = Math.min(left, x);
      right = Math.max(right, x + 1);
      top = Math.min(top, y);
      bottom = Math.max(bottom, y + 1);
    }
  }
  let [weight, weightedX] = [0, 0];
  for (let y = Math.round((top + bottom) / 2); y < bottom; y += 1) {
    for (let x = left; x < right; x += 1) {
      weight += alphaAt(x, y);
      weightedX += alphaAt(x, y) * (x + 0.5);
    }
  }
  const toIcon = (value) => value / MEASURE_ZOOM - pad;
  return {
    bottom: toIcon(bottom),
    left: toIcon(left),
    lowerHalfX: toIcon(weightedX / weight),
    right: toIcon(right),
    top: toIcon(top),
  };
}

/** The tray mark's arc with `glyph` drawn over it, as a PNG. */
const iconWith = (glyph) =>
  new Resvg(
    `<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="${SIZE}" height="${SIZE}">
  <clipPath id="arc"><rect x="0" y="${ARC_TOP}" width="${SIZE}" height="${SIZE - ARC_TOP}"/></clipPath>
  <image clip-path="url(#arc)" width="${SIZE}" height="${SIZE}" xlink:href="data:image/png;base64,${arc}"/>
  ${glyph}
</svg>`,
    { font: fontOptions },
  )
    .render()
    .asPng();

for (let seconds = 1; seconds <= LONGEST_DELAY; seconds += 1) {
  // resvg draws the variable font at its default weight, so a stroke gives
  // the number the weight of the arc. Two digits sit closer to fit the box.
  const spacing = seconds < 10 ? 0 : -1.5;
  const number = `<text x="0" y="0" font-family="Roboto Mono" font-size="20" letter-spacing="${spacing}" fill="${INK}" stroke="${INK}" stroke-width="2.4" stroke-linejoin="round">${seconds}</text>`;
  const ink = measureInk(number);
  const scale = Math.min(
    GLYPH_HEIGHT / (ink.bottom - ink.top),
    GLYPH_WIDTH / (ink.right - ink.left),
  );
  // Centred on its bounds, except a lone "1": its flag reaches out to one
  // side, so its bounds would leave the stem visibly off centre. Its lower
  // half is the stem alone.
  const centreX = seconds === 1 ? ink.lowerHalfX : (ink.left + ink.right) / 2;
  const x = GLYPH_CENTRE[0] - centreX * scale;
  const y = GLYPH_CENTRE[1] - ((ink.top + ink.bottom) / 2) * scale;
  const png = iconWith(
    `<g transform="translate(${x} ${y}) scale(${scale})">${number}</g>`,
  );
  await writeFile(new URL(`${seconds}.png`, output), png);
}

const [centreX, centreY] = GLYPH_CENTRE;
const history = renderToStaticMarkup(
  createElement(History, {
    absoluteStrokeWidth: true,
    color: INK,
    size: HISTORY_SIZE,
    strokeWidth: HISTORY_STROKE,
    x: centreX - HISTORY_SIZE / 2,
    y: centreY - HISTORY_SIZE / 2,
  }),
);
await writeFile(new URL("tray-replay.png", icons), iconWith(history));
