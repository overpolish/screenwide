// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { annotationLaneLabel } from "./annotation-kinds";
import {
  Annotation,
  annotationDeleteTarget,
  renumberedCounters,
  validAnnotations,
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
  style: { align: "left", color: "#ff383c", head: "end", width: 8 },
});

describe("annotationDeleteTarget", () => {
  const annotations = [arrow("a"), arrow("b")];

  it("takes the annotation the hand is pointing at over the chosen one", () => {
    expect(annotationDeleteTarget(annotations, "b", "a")).toBe("b");
  });

  it("falls back to the chosen annotation when nothing is hovered", () => {
    expect(annotationDeleteTarget(annotations, null, "a")).toBe("a");
  });

  it("ignores a hover on an annotation this layer no longer carries", () => {
    expect(annotationDeleteTarget(annotations, "gone", "a")).toBe("a");
  });

  it("has nothing to delete when neither names an annotation", () => {
    expect(annotationDeleteTarget(annotations, null, null)).toBeNull();
    expect(annotationDeleteTarget([], "a", "b")).toBeNull();
  });
});

const counter = (id: string, value: number): Annotation => ({
  aboveCamera: false,
  animated: true,
  id,
  shape: { angle: 0, center: { x: 20, y: 30 }, kind: "counter", value },
  style: { align: "left", color: "#ff383c", head: "none", width: 56 },
});

describe("renumberedCounters", () => {
  it("closes the gap a deleted counter leaves", () => {
    const renumbered = renumberedCounters([counter("a", 1), counter("c", 3)]);
    expect(renumbered.map((annotation) => annotation.shape)).toEqual([
      expect.objectContaining({ value: 1 }),
      expect.objectContaining({ value: 2 }),
    ]);
  });

  it("numbers by place in the list, arrows in between and all", () => {
    const renumbered = renumberedCounters([
      counter("a", 9),
      arrow("b"),
      counter("c", 9),
    ]);
    expect(renumbered.map((annotation) => annotation.shape.kind)).toEqual([
      "counter",
      "arrow",
      "counter",
    ]);
    expect(
      renumbered
        .map((annotation) => annotation.shape)
        .filter((shape) => shape.kind === "counter")
        .map((shape) => shape.value),
    ).toEqual([1, 2]);
  });

  it("hands back the very same list when nothing moved", () => {
    const annotations = [counter("a", 1), counter("b", 2)];
    expect(renumberedCounters(annotations)).toBe(annotations);
  });
});

describe("validAnnotations", () => {
  it("reads a stored counter back whole", () => {
    expect(validAnnotations([counter("a", 2)])).toEqual([counter("a", 2)]);
  });

  it("drops a counter the compositor could not place", () => {
    const broken = (shape: Record<string, unknown>) => [
      { ...counter("a", 1), shape: { kind: "counter", ...shape } },
    ];
    expect(
      validAnnotations(broken({ angle: 0, center: { x: 1, y: 1 } })),
    ).toEqual([]);
    expect(
      validAnnotations(
        broken({ angle: Number.NaN, center: { x: 1, y: 1 }, value: 1 }),
      ),
    ).toEqual([]);
    expect(
      validAnnotations(broken({ angle: 0, center: { x: 1, y: 1 }, value: 0 })),
    ).toEqual([]);
    expect(validAnnotations(broken({ angle: 0, value: 1 }))).toEqual([]);
  });

  it("ignores a shape this build does not know", () => {
    expect(
      validAnnotations([{ ...counter("a", 1), shape: { kind: "blob" } }]),
    ).toEqual([]);
  });

  it("reads a stored text box back, tucking in a pointer it cannot read", () => {
    const box = (pointer: unknown) => ({
      ...counter("t", 1),
      shape: { kind: "text", origin: { x: 4, y: 5 }, pointer, text: "Hi" },
      style: { align: "center", color: "#ffcc00", head: "none", width: 28 },
    });
    const out = { along: { x: 1, y: -0.5 }, reach: { x: 3, y: 0 } };
    const [read] = validAnnotations([box(out)]);
    expect(read.shape).toEqual({
      kind: "text",
      origin: { x: 4, y: 5 },
      pointer: out,
      text: "Hi",
    });
    expect(read.style.align).toBe("center");
    const tucked = { along: { x: 0, y: 1 }, reach: { x: 0, y: 0 } };
    for (const unreadable of [null, { x: 9, y: 9 }, { along: out.along }])
      expect(validAnnotations([box(unreadable)])[0].shape).toMatchObject({
        pointer: tucked,
      });
  });

  it("reads an older document's style as left-aligned", () => {
    const [read] = validAnnotations([
      {
        ...counter("a", 1),
        style: { color: "#ffcc00", head: "none", width: 56 },
      },
    ]);
    expect(read.style.align).toBe("left");
  });
});

describe("annotationLaneLabel", () => {
  it("calls a text box by its first line, and by its place when that is empty", () => {
    const box = (text: string): Annotation => ({
      ...counter("t", 1),
      shape: {
        kind: "text",
        origin: { x: 0, y: 0 },
        pointer: { along: { x: 0, y: 1 }, reach: { x: 0, y: 0 } },
        text,
      },
    });
    expect(annotationLaneLabel(box("Save here\nthen quit"), 2)).toBe(
      "Save here",
    );
    expect(annotationLaneLabel(box("\nsecond"), 2)).toBe("Text 3");
  });
});
