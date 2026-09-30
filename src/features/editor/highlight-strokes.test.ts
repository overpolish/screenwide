// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { highlightStrokes, relaidHighlight } from "./highlight-strokes";

import type { Annotation, AnnotationStyle } from "./annotations";

const style: AnnotationStyle = {
  align: "left",
  blur: false,
  color: "#ffcc00",
  handDrawn: false,
  head: "none",
  manual: false,
  radius: 0,
  redaction: "erase",
  shadow: false,
  softness: 0,
  strength: 0,
  width: 24,
};

/** A highlight from (10, 100) to (200, 300), fitted to two lines of text
 * 20 pixels tall. */
const fitted: Annotation = {
  aboveCamera: false,
  animated: true,
  id: "h",
  shape: {
    bands: [
      { bottom: 120, left: 10, right: 200, top: 100 },
      { bottom: 300, left: 10, right: 90, top: 280 },
    ],
    end: { x: 200, y: 300 },
    kind: "highlight",
    seed: 1,
    start: { x: 10, y: 100 },
    tone: { ink: 0, surface: 1 },
  },
  style,
};

describe("highlightStrokes", () => {
  it("lays a drag about one stroke tall as one stroke centred on it", () => {
    expect(highlightStrokes({ x: 10, y: 100 }, { x: 200, y: 110 }, 24)).toEqual(
      [{ bottom: 117, left: 10, right: 200, top: 93 }],
    );
  });

  it("covers a box edge to edge with overlapping strokes", () => {
    const strokes = highlightStrokes({ x: 200, y: 300 }, { x: 10, y: 100 }, 24);
    expect(strokes.length).toBeGreaterThan(1);
    expect(strokes[0]?.top).toBe(100);
    expect(strokes[strokes.length - 1]?.bottom).toBe(300);
    for (let index = 1; index < strokes.length; index += 1) {
      const overlap = strokes[index - 1].bottom - strokes[index].top;
      expect(overlap).toBeGreaterThanOrEqual(24 * 0.08 - 1e-9);
      expect(overlap).toBeLessThan(12);
    }
  });
});

describe("relaidHighlight", () => {
  it("lays a fitted highlight switched to manual as a tinted box of its first line's height", () => {
    const shape = relaidHighlight(fitted, { ...style, manual: true });
    const strokes = shape?.bands ?? [];
    expect(strokes.length).toBeGreaterThan(2);
    for (const stroke of strokes) {
      expect(stroke.bottom - stroke.top).toBeCloseTo(20);
      expect([stroke.left, stroke.right]).toEqual([10, 200]);
    }
    expect(shape?.tone.ink).toBe(shape?.tone.surface);
  });

  it("leaves a highlight alone unless it is switched to manual", () => {
    expect(relaidHighlight(fitted, { ...style, color: "#ff0000" })).toBeNull();
    const manual = { ...fitted, style: { ...style, manual: true } };
    expect(
      relaidHighlight(manual, { ...style, color: "#ff0000", manual: true }),
    ).toBeNull();
  });
});
