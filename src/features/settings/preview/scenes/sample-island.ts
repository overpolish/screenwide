// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ChevronDown,
  Camera,
  Circle,
  ImageDown,
  type LucideIcon,
  Mic,
  Volume2,
} from "lucide-react";

import {
  ISLAND,
  type IslandNotch,
} from "../../../../primitives/island/island-geometry";

import { paintGlyph } from "./paint-glyph";
import { paintNotch } from "./sample-system-bars";
import { paintWallpaper, type Painter, token } from "./scene-kit";

// Screenwide's island as it rests at the top of a display, painted into a
// recording scene's picture: the recording island's row at rest, in its
// own sizes, tokens and glyphs, so the scenes that show Screenwide in a
// recording show it as it looks rather than a stand-in shape. The row is
// `RestingRow`: what is recorded, the three inputs, then the screenshot and
// Record, each with its menu, around the notch on a display with one.

/** The island's controls: their height, their glyph and its menu. */
const CONTROL = 32;
const GLYPH = 16;
const MENU = { gap: 2, glyph: 12, width: 14 };
const RECORD_WIDTH = 48;
/** The space between neighbours, `gap-separate`. */
const SEPARATE = 12;
/** Between the black's edge and the row's controls. */
const INSET = 8;

const INPUTS: LucideIcon[] = [Mic, Camera, Volume2];

/** The island's dark skin, read from the tokens it is drawn with. */
function islandSkin() {
  const probe = document.createElement("div");
  probe.className = "dark";
  document.body.append(probe);
  const skin = {
    black: token("island", probe),
    fg: token("fg", probe),
    menu: token("fg-tertiary", probe),
    onPrimary: token("on-primary", probe),
    primary: token("primary", probe),
    shadow: getComputedStyle(probe).getPropertyValue("--island-shadow"),
  };
  probe.remove();
  return skin;
}

/** The island's two drop shadows (`--island-shadow`), as offsets, blur and
 * colour. */
function shadowsOf(filter: string) {
  return [
    ...filter.matchAll(
      /drop-shadow\(\s*([\d.-]+)(?:px)?\s+([\d.-]+)(?:px)?\s+([\d.-]+)(?:px)?\s+(rgba?\([^)]*\)|[^)]+)\)/g,
    ),
  ].map(([, x, y, blur, colour]) => ({
    blur: Number(blur),
    colour: colour.trim(),
    x: Number(x),
    y: Number(y),
  }));
}

const controlWidth = CONTROL + MENU.gap + MENU.width;
/** The row's halves: the source and three inputs, then the screenshot and
 * Record. */
const START_WIDTH = CONTROL + (SEPARATE + controlWidth) * INPUTS.length;
const END_WIDTH =
  controlWidth + SEPARATE + RECORD_WIDTH + MENU.gap + MENU.width;
const ROW_HEIGHT = CONTROL + INSET * 2;

/** Where the island's black runs, from its start to its end, and how far
 * down it hangs. Without a notch the halves sit a neighbour's space apart
 * in the middle of the screen, hanging just below its top. Around a notch
 * they are parted by it, each as wide as it holds, so the island is
 * shifted towards the narrower half to keep the notch over the gap, and it
 * runs up into the screen's edge. */
function islandBox(width: number, notch: IslandNotch | null) {
  if (!notch) {
    const row = INSET * 2 + START_WIDTH + SEPARATE + END_WIDTH;
    const left = Math.round((width - row) / 2);
    return { left, right: left + row, top: ISLAND.screenGap };
  }
  const room = notch.width / 2 + ISLAND.notchGap;
  return {
    left: Math.round(width / 2 - room - START_WIDTH - INSET),
    right: Math.round(width / 2 + room + END_WIDTH + INSET),
    top: 0,
  };
}

/** The black under a notch: flush with the screen's top, its corners there
 * flaring out into the edge, its bottom corners a little tighter. */
function notchShape(
  context: CanvasRenderingContext2D,
  { left, right }: { left: number; right: number },
) {
  const flare = ISLAND.flare;
  const radius = ISLAND.notchRadius;
  const bottom = ROW_HEIGHT;
  context.moveTo(left - flare, 0);
  context.lineTo(right + flare, 0);
  context.quadraticCurveTo(right, 0, right, flare);
  context.lineTo(right, bottom - radius);
  context.arcTo(right, bottom, right - radius, bottom, radius);
  context.lineTo(left + radius, bottom);
  context.arcTo(left, bottom, left, bottom - radius, radius);
  context.lineTo(left, flare);
  context.quadraticCurveTo(left, 0, left - flare, 0);
  context.closePath();
}

export const paintIsland: Painter = (
  context,
  { width },
  { notch, wallpaper },
) => {
  const skin = islandSkin();
  const box = islandBox(width, notch);
  const { left, right, top } = box;
  const middle = top + ROW_HEIGHT / 2;
  context.save();
  for (const shadow of shadowsOf(skin.shadow)) {
    context.shadowBlur = shadow.blur;
    context.shadowColor = shadow.colour;
    context.shadowOffsetX = shadow.x;
    context.shadowOffsetY = shadow.y;
    context.fillStyle = skin.black;
    context.beginPath();
    if (notch) notchShape(context, box);
    else context.roundRect(left, top, right - left, ROW_HEIGHT, ISLAND.radius);
    context.fill();
  }
  context.restore();
  // The notch is the display's own, over the island's black.
  if (notch) paintNotch(context, width, notch);

  // What is recorded: the display, as its picture in a circle.
  let x = left + INSET;
  context.save();
  context.beginPath();
  context.arc(x + CONTROL / 2, middle, CONTROL / 2, 0, Math.PI * 2);
  context.clip();
  context.translate(x, middle - CONTROL / 2);
  paintWallpaper(context, { height: CONTROL, width: CONTROL }, wallpaper);
  context.restore();
  x += CONTROL + SEPARATE;

  const control = (icon: LucideIcon, at: number) => {
    paintGlyph(context, icon, {
      colour: skin.fg,
      size: GLYPH,
      x: at + (CONTROL - GLYPH) / 2,
      y: middle - GLYPH / 2,
    });
  };
  const menu = (at: number) => {
    paintGlyph(context, ChevronDown, {
      colour: skin.menu,
      size: MENU.glyph,
      x: at + MENU.gap + (MENU.width - MENU.glyph) / 2,
      y: middle - MENU.glyph / 2,
    });
  };
  for (const icon of INPUTS) {
    control(icon, x);
    menu(x + CONTROL);
    x += controlWidth + SEPARATE;
  }

  // From the end: Record's menu, Record, then the screenshot.
  let end = right - INSET - MENU.width - MENU.gap;
  menu(end);
  end -= RECORD_WIDTH;
  context.fillStyle = skin.primary;
  context.beginPath();
  context.roundRect(
    end,
    middle - CONTROL / 2,
    RECORD_WIDTH,
    CONTROL,
    CONTROL / 2,
  );
  context.fill();
  paintGlyph(context, Circle, {
    colour: skin.onPrimary,
    size: GLYPH,
    x: end + (RECORD_WIDTH - GLYPH) / 2,
    y: middle - GLYPH / 2,
  });
  end -= SEPARATE + controlWidth;
  control(ImageDown, end);
  menu(end + CONTROL);
};
