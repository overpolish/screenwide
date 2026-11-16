// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { placeShortcutKeys } from "./shortcut-placement";

describe("placeShortcutKeys", () => {
  it("sets keys pressed close together side by side", () => {
    expect(
      placeShortcutKeys(
        [
          { keysPx: 40, x: 100 },
          { keysPx: 40, x: 110 },
        ],
        640,
      ),
    ).toEqual([100, 150]);
  });

  it("ends keys pressed in the last moments at the strip's end", () => {
    expect(
      placeShortcutKeys(
        [
          { keysPx: 40, x: 590 },
          { keysPx: 40, x: 630 },
        ],
        640,
      ),
    ).toEqual([550, 600]);
  });

  it("keeps every key on a strip too full to hold them", () => {
    const places = Array.from({ length: 30 }, (_, index) => ({
      keysPx: 40,
      x: 100 + index * 5,
    }));
    const starts = placeShortcutKeys(places, 640);
    expect(Math.min(...starts)).toBe(0);
    expect(starts[starts.length - 1]).toBe(600);
  });
});
