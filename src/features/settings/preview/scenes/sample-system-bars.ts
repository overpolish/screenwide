// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isWindows } from "../../../../lib/platform";
import {
  ISLAND,
  type IslandNotch,
} from "../../../../primitives/island/island-geometry";

import { SAMPLE } from "./sample-palette";

/** The menu bar's height without a notch; with one, the bar is as tall as
 * the notch, as macOS draws it. Windows has no bar at the top. */
const MENU_BAR = 24;
/** How far the Dock's room reaches up from the screen's bottom, and the
 * Dock's own height inside it. */
const DOCK_ROOM = 62;
const DOCK = 52;
/** Windows' taskbar along the bottom. */
const TASKBAR = 40;

export const menuBarHeight = (notch: IslandNotch | null) =>
  isWindows() ? 0 : notch ? notch.height : MENU_BAR;

/** macOS's menu bar along the top of a screen `width` wide: the menus at
 * its start, the status items at its end, and on a Mac with one, the notch
 * in its middle, black from the screen's edge. */
export function paintMenuBar(
  context: CanvasRenderingContext2D,
  width: number,
  notch: IslandNotch | null,
) {
  const height = menuBarHeight(notch);
  const middle = height / 2;
  context.fillStyle = SAMPLE.menuBar;
  context.fillRect(0, 0, width, height);
  context.fillStyle = SAMPLE.menuBarInk;
  let x = 16;
  for (const item of [30, 26, 34, 30, 44]) {
    context.beginPath();
    context.roundRect(x, middle - 3, item, 6, 3);
    context.fill();
    x += item + 14;
  }
  context.beginPath();
  context.roundRect(width - 70, middle - 3, 54, 6, 3);
  context.arc(width - 92, middle, 4, 0, Math.PI * 2);
  context.fill();
  if (notch) paintNotch(context, width, notch);
}

/** The display's notch in the middle of the top of a screen `width` wide,
 * the bezel's black over whatever is under it. */
export function paintNotch(
  context: CanvasRenderingContext2D,
  width: number,
  notch: IslandNotch,
) {
  context.fillStyle = SAMPLE.notch;
  context.beginPath();
  context.roundRect((width - notch.width) / 2, 0, notch.width, notch.height, [
    0,
    0,
    ISLAND.hiddenNotchRadius,
    ISLAND.hiddenNotchRadius,
  ]);
  context.fill();
}

/** The system's own bars over the sample desktop: macOS's menu bar along
 * the top, around the notch where the display has one, or Windows' taskbar
 * along the bottom. */
export function paintSystemBars(
  context: CanvasRenderingContext2D,
  { height, width }: { height: number; width: number },
  notch: IslandNotch | null,
) {
  if (!isWindows()) {
    paintMenuBar(context, width, notch);
    return;
  }
  const bar = TASKBAR;
  context.fillStyle = SAMPLE.taskbar;
  context.fillRect(0, height - bar, width, bar);
  const icon = 22;
  const gap = 12;
  const span = SAMPLE.icons.length * (icon + gap) - gap;
  SAMPLE.icons.forEach((colour, index) => {
    context.fillStyle = colour;
    context.beginPath();
    context.roundRect(
      (width - span) / 2 + index * (icon + gap),
      height - bar + (bar - icon) / 2,
      icon,
      icon,
      6,
    );
    context.fill();
  });
}

/** macOS's Dock along the bottom of a screen `size` big, its apps in a row
 * on glass. Windows has no Dock. */
export function paintDock(
  context: CanvasRenderingContext2D,
  { height, width }: { height: number; width: number },
) {
  if (isWindows()) return;
  const icon = 38;
  const gap = 10;
  const span = SAMPLE.icons.length * (icon + gap) - gap + 2 * gap;
  const top = height - DOCK_ROOM + (DOCK_ROOM - DOCK) / 2;
  context.fillStyle = SAMPLE.menuBar;
  context.beginPath();
  context.roundRect((width - span) / 2, top, span, DOCK, 16);
  context.fill();
  SAMPLE.icons.forEach((colour, index) => {
    context.fillStyle = colour;
    context.beginPath();
    context.roundRect(
      (width - span) / 2 + gap + index * (icon + gap),
      top + (DOCK - icon) / 2,
      icon,
      icon,
      9,
    );
    context.fill();
  });
}

/** What a screen recording keeps of a screen `size` big with the system's
 * bars painted on it as above, with the menu bar (the taskbar on Windows)
 * and the Dock left out where chosen, in its points. */
export function keptOfScreen(
  size: { height: number; width: number },
  notch: IslandNotch | null,
  { dock, menuBar }: { dock: boolean; menuBar: boolean },
) {
  const top = menuBar ? menuBarHeight(notch) : 0;
  const bottom = isWindows() ? (menuBar ? TASKBAR : 0) : dock ? DOCK_ROOM : 0;
  return {
    height: size.height - top - bottom,
    width: size.width,
    x: 0,
    y: top,
  };
}
