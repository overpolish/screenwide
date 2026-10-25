// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  type Box,
  paintCard,
  paintWallpaper,
  type Spot,
  type ToolScene,
  travel,
} from "./scene-kit";

// Ruler: two windows, the gap between them read by hovering, then one of
// them dragged over and measured, its box snapped to the window's edges.

function rulerLayout(width: number) {
  const first: Box = {
    height: 104,
    width: 150,
    x: Math.round(width / 2 - 190),
    y: 28,
  };
  const second: Box = {
    height: 64,
    width: 132,
    x: first.x + first.width + 56,
    y: 48,
  };
  return { first, second };
}

export const RULER_SCENE: ToolScene = {
  duration: 8,
  paint(context, width, height) {
    paintWallpaper(context, width, height);
    const { first, second } = rulerLayout(width);
    paintCard(context, first, 10);
    paintCard(context, second, 10);
  },
  pointer(at, width) {
    const { first, second } = rulerLayout(width);
    const gap = first.x + first.width + (second.x - first.x - first.width) / 2;
    const middle = first.y + first.height / 2;
    const corner: Spot = [second.x - 8, second.y - 8];
    const across: Spot = [
      second.x + second.width + 8,
      second.y + second.height + 8,
    ];
    const inside: Spot = [
      first.x + first.width * 0.4,
      first.y + first.height * 0.45,
    ];
    if (at < 1.8) {
      return { pressed: false, x: gap, y: middle + Math.sin(at * 2.4) * 22 };
    }
    if (at < 2.3) {
      return {
        pressed: false,
        ...travel(at, [1.8, 2.3], [[gap, middle], corner]),
      };
    }
    if (at < 3.4) {
      return { pressed: true, ...travel(at, [2.4, 3.3], [corner, across]) };
    }
    if (at < 4.2) {
      return { pressed: false, ...travel(at, [3.4, 4.2], [across, inside]) };
    }
    if (at < 7.6) {
      const drift = at - 4.2;
      return {
        pressed: false,
        x: inside[0] + Math.sin(drift * 1.3) * 24,
        y: inside[1] + Math.sin(drift * 0.9) * 14,
      };
    }
    return null;
  },
};
