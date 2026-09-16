// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { Annotation, annotationDeleteTarget } from "./annotations";

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
