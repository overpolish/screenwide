// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { withAnnotationColor, withoutAnnotationColor } from "./palette";

describe("the colours of your own", () => {
  it("keeps a colour the palette does not offer, newest last", () => {
    expect(withAnnotationColor(["#2ec4b6"], "#8b5e34")).toEqual([
      "#2ec4b6",
      "#8b5e34",
    ]);
  });

  it("does not keep one the palette already offers", () => {
    expect(withAnnotationColor([], "#FF383C")).toEqual([]);
  });

  it("moves a colour already kept to the end rather than repeating it", () => {
    expect(withAnnotationColor(["#2ec4b6", "#8b5e34"], "#2EC4B6")).toEqual([
      "#8b5e34",
      "#2ec4b6",
    ]);
  });

  it("forgets the oldest past the cap", () => {
    let kept: string[] = [];
    for (let index = 0; index < 20; index++)
      kept = withAnnotationColor(
        kept,
        `#0000${index.toString(16).padStart(2, "0")}`,
      );
    expect(kept).toHaveLength(12);
    expect(kept[kept.length - 1]).toBe("#000013");
  });

  it("ignores something that is not a colour", () => {
    expect(withAnnotationColor([], "rebeccapurple")).toEqual([]);
  });

  it("forgets a colour whatever case it is asked for in", () => {
    expect(withoutAnnotationColor(["#2ec4b6", "#8b5e34"], "#2EC4B6")).toEqual([
      "#8b5e34",
    ]);
  });
});
