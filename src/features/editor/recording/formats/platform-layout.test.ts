// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { platformClearArea } from "./platform-clear-area";
import { laidScreen, type ScreenBox, screenIntent } from "./platform-layout";

import type { ClearArea } from "../../../../bindings/ClearArea";

const WIDE = { height: 1_080, width: 1_920 };
const SQUARE = { height: 1_200, width: 1_200 };
const TALL = { height: 1_920, width: 1_080 };
const QHD = { height: 1_440, width: 2_560 };
const TIKTOK = platformClearArea(["tiktok"]);

type Size = { height: number; width: number };

/** A screen of 16:10 on `canvas`, `share` of its width wide, its empty
 * space split `across` and `down`. */
const screen = (
  canvas: Size,
  share: number,
  [across, down]: [number, number],
): ScreenBox => {
  const width = canvas.width * share;
  const height = width / 1.6;
  return {
    height,
    width,
    x: across * (canvas.width - width),
    y: down * (canvas.height - height),
  };
};

/** `box` on a canvas `from` big laid out on one `to` big, clear of
 * `clear`. */
const laid = (
  from: Size,
  box: ScreenBox,
  { clear, to }: { to: Size; clear?: ClearArea },
) => laidScreen(to, screenIntent(from, box), clear);

describe("platform layout", () => {
  it("keeps a centred screen centred and inside TikTok's clear area", () => {
    const box = laid(WIDE, screen(WIDE, 0.82, [0.5, 0.5]), {
      clear: TIKTOK,
      to: TALL,
    });
    const top = TALL.height * 0.07;
    const bottom = TALL.height * 0.75;
    expect(box.x + box.width / 2).toBeCloseTo(TALL.width / 2);
    expect(box.y + box.height / 2).toBeCloseTo((top + bottom) / 2);
    expect(box.x).toBeGreaterThanOrEqual(TALL.width * 0.05 - 1e-9);
  });

  it("holds a full-width screen at the top of the clear area, edge to edge", () => {
    const box = laid(SQUARE, screen(SQUARE, 1, [0.5, 0]), {
      clear: TIKTOK,
      to: TALL,
    });
    expect(box.x).toBeCloseTo(0);
    expect(box.width).toBeCloseTo(TALL.width);
    expect(box.y).toBeCloseTo(TALL.height * 0.07);
  });

  it("lets go of the top on a canvas it can no longer span", () => {
    const box = laid(SQUARE, screen(SQUARE, 1, [0.5, 0]), { to: QHD });
    expect(box.y).toBeGreaterThan(0);
    expect(box.y + box.height).toBeLessThan(QHD.height);
    expect(box.x + box.width / 2).toBeCloseTo(QHD.width / 2);
  });

  it("runs a screen cut off at the bottom past the clear area's bottom", () => {
    const width = WIDE.width * 0.7;
    const cut = { height: width / 1.6, width, x: 288, y: 324 };
    const box = laid(WIDE, cut, { clear: TIKTOK, to: TALL });
    const shown = (WIDE.height - cut.y) / cut.height;
    expect(box.y + box.height * shown).toBeCloseTo(TALL.height * 0.75);
  });

  it("leans a screen in a corner towards the middle on a vertical app", () => {
    const box = laid(WIDE, screen(WIDE, 0.4, [0.92, 0.9]), {
      clear: TIKTOK,
      to: TALL,
    });
    const top = TALL.height * 0.07;
    const bottom = TALL.height * 0.75;
    const down = (box.y - top) / (bottom - top - box.height);
    expect(down).toBeGreaterThan(0.5);
    expect(down).toBeLessThan(0.9);
  });

  it("grows a small screen on a tall main canvas to its weight on a wide one", () => {
    const tall = { height: 1_800, width: 999 };
    const box = laid(tall, screen(tall, 0.45, [0.38, 0.2]), { to: QHD });
    // Fitted into the wide canvas whole it would be 449 wide.
    expect(box.width).toBeGreaterThan(900);
  });
});
