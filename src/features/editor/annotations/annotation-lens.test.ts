// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { validAnnotations } from "./annotation-documents";
import {
  LENS_ZOOMS,
  NEW_LENS_ZOOM,
  nearestLensZoom,
} from "./annotation-shape-parsers";

/** A stored lens over (10, 10)-(90, 90), with `shape` laid over it. */
const stored = (shape: Record<string, unknown>) => ({
  animated: true,
  id: "lens",
  shape: {
    end: { x: 90, y: 90 },
    kind: "magnify",
    start: { x: 10, y: 10 },
    ...shape,
  },
  style: {
    align: "left",
    blur: false,
    color: "#ffffff",
    handDrawn: false,
    head: "none",
    manual: false,
    radius: 50,
    redaction: "erase",
    shadow: true,
    softness: 0,
    strength: 0,
    tint: false,
    width: 8,
  },
});

describe("lenses", () => {
  it("reads a stored lens back, and drops one it cannot draw", () => {
    const [read] = validAnnotations([stored({ zoom: 3 })]);
    expect(read.shape).toEqual({
      end: { x: 90, y: 90 },
      kind: "magnify",
      start: { x: 10, y: 10 },
      zoom: 3,
    });
    expect(validAnnotations([stored({ zoom: 0 })])).toEqual([]);
    expect(validAnnotations([stored({ start: undefined, zoom: 2 })])).toEqual(
      [],
    );
    // One with no zoom at all takes a fresh lens's.
    expect(validAnnotations([stored({})])[0].shape).toEqual(
      expect.objectContaining({ zoom: NEW_LENS_ZOOM }),
    );
  });

  it("reads a magnifier from before the lens as a lens over its zoom area", () => {
    // An 80-wide zoom area whose loupe was 200 across showed it 2.5 times:
    // the lens is 200 across over its centre, at the nearer look, 3×.
    const [read] = validAnnotations([
      stored({
        end: { x: 90, y: 50 },
        loupe: { x: 300, y: 50 },
        size: 200,
      }),
    ]);
    expect(read.shape).toEqual({
      end: { x: 150, y: 80 },
      kind: "magnify",
      start: { x: -50, y: -20 },
      zoom: 3,
    });
  });

  it("snaps a zoom to the nearest look as a ratio", () => {
    expect(LENS_ZOOMS.map(nearestLensZoom)).toEqual([...LENS_ZOOMS]);
    expect(nearestLensZoom(2.5)).toBe(3);
    expect(nearestLensZoom(1.7)).toBe(1.5);
    expect(nearestLensZoom(10)).toBe(4);
    expect(nearestLensZoom(Number.NaN)).toBe(NEW_LENS_ZOOM);
  });
});
