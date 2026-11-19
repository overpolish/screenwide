// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  AnnotationClipboard,
  parsedAnnotationClipboard,
  pastedAnnotations,
} from "./annotation-clipboard";
import { Annotation } from "./annotations";

const arrow = (from: number, to: number): Annotation => ({
  animated: true,
  id: "a",
  shape: {
    control: { x: (from + to) / 2, y: 100 },
    end: { x: to, y: 100 },
    kind: "arrow",
    start: { x: from, y: 100 },
  },
  style: {
    align: "left",
    blur: false,
    color: "#ff383c",
    handDrawn: false,
    head: "end",
    manual: false,
    radius: 0,
    redaction: "erase",
    shadow: false,
    softness: 0,
    strength: 0,
    tint: false,
    width: 8,
  },
});

const retina = { height: 1_000, scaleFactor: 2, width: 2_000 };
const copied: AnnotationClipboard = {
  capture: retina,
  items: [{ annotation: arrow(100, 300), lengthMs: 3_000 }],
};

const startOf = (annotation: Annotation) =>
  annotation.shape.kind === "arrow" ? annotation.shape.start : null;

describe("pasting annotations", () => {
  it("lands where they were on a picture that has none of them", () => {
    const [pasted] = pastedAnnotations(copied, retina, []);
    expect(startOf(pasted)).toEqual({ x: 100, y: 100 });
    expect(pasted.id).not.toBe("a");
  });

  it("steps clear of the originals, and of each paste before", () => {
    const [first] = pastedAnnotations(copied, retina, [arrow(100, 300)]);
    // 16 points at two pixels to the point.
    expect(startOf(first)).toEqual({ x: 132, y: 132 });
    const [second] = pastedAnnotations(copied, retina, [
      arrow(100, 300),
      first,
    ]);
    expect(startOf(second)).toEqual({ x: 164, y: 164 });
  });

  it("keeps their size in points and their place as a share of another picture", () => {
    // A picture half as wide in pixels, at one pixel to the point.
    const [pasted] = pastedAnnotations(
      copied,
      { height: 500, scaleFactor: 1, width: 1_000 },
      [],
    );
    if (pasted.shape.kind !== "arrow") throw new Error("an arrow");
    // 200 pixels at 2x is 100 points: 100 pixels at 1x, about the middle
    // the arrow had, a tenth of the way across.
    expect(pasted.shape.end.x - pasted.shape.start.x).toBeCloseTo(100);
    expect((pasted.shape.start.x + pasted.shape.end.x) / 2).toBeCloseTo(100);
  });
});

describe("the clipboard as read back", () => {
  it("keeps what is usable and drops the rest", () => {
    expect(parsedAnnotationClipboard(null)).toBeNull();
    expect(
      parsedAnnotationClipboard({ capture: retina, items: [{ junk: 1 }] }),
    ).toBeNull();
    expect(parsedAnnotationClipboard(copied)).toEqual(copied);
  });
});
