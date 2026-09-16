// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  withAnnotationColor,
  withoutAnnotationColor,
} from "./annotation-palette";
import {
  ANNOTATION_WIDTHS,
  Annotation,
  annotationDeleteTarget,
  annotationWidthAt,
  annotationWidthIndex,
  DEFAULT_ANNOTATION_WIDTH,
} from "./annotations";

const arrow = (id: string): Annotation => ({
  aboveCamera: false,
  animated: true,
  id,
  shape: {
    control: { x: 5, y: 0 },
    end: { x: 10, y: 0 },
    kind: "arrow",
    start: { x: 0, y: 0 },
  },
  style: { color: "#ff383c", head: "end", width: 8 },
});

describe("annotationDeleteTarget", () => {
  const annotations = [arrow("a"), arrow("b")];

  it("takes the mark the hand is pointing at over the chosen one", () => {
    expect(annotationDeleteTarget(annotations, "b", "a")).toBe("b");
  });

  it("falls back to the chosen mark when nothing is hovered", () => {
    expect(annotationDeleteTarget(annotations, null, "a")).toBe("a");
  });

  it("ignores a hover on a mark this layer no longer carries", () => {
    expect(annotationDeleteTarget(annotations, "gone", "a")).toBe("a");
  });

  it("has nothing to delete when neither names a mark", () => {
    expect(annotationDeleteTarget(annotations, null, null)).toBeNull();
    expect(annotationDeleteTarget([], "a", "b")).toBeNull();
  });
});

describe("the width presets", () => {
  it("maps every preset to its own place and back", () => {
    for (const [index, width] of ANNOTATION_WIDTHS.entries()) {
      expect(annotationWidthIndex(width)).toBe(index);
      expect(annotationWidthAt(index)).toBe(width);
    }
  });

  it("lands a width from an older document on the nearest preset", () => {
    expect(annotationWidthIndex(2)).toBe(0);
    expect(annotationWidthIndex(13)).toBe(ANNOTATION_WIDTHS.indexOf(12));
    expect(annotationWidthIndex(100)).toBe(ANNOTATION_WIDTHS.length - 1);
  });

  it("keeps the tool's own default when the width is not a number", () => {
    expect(annotationWidthAt(annotationWidthIndex(Number.NaN))).toBe(
      DEFAULT_ANNOTATION_WIDTH,
    );
  });

  it("clamps an index the slider could never report", () => {
    expect(annotationWidthAt(-3)).toBe(ANNOTATION_WIDTHS[0]);
    expect(annotationWidthAt(99)).toBe(
      ANNOTATION_WIDTHS[ANNOTATION_WIDTHS.length - 1],
    );
  });
});

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
