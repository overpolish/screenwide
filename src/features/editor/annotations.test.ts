// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { validAnnotations } from "./annotation-documents";
import { annotationLaneLabel } from "./annotation-kinds";
import { Annotation, renumberedCounters } from "./annotations";

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
    width: 8,
  },
});

const counter = (id: string, value: number): Annotation => ({
  aboveCamera: false,
  animated: true,
  id,
  shape: { angle: 0, center: { x: 20, y: 30 }, kind: "counter", value },
  style: {
    align: "left",
    blur: false,
    color: "#ff383c",
    handDrawn: false,
    head: "none",
    manual: false,
    radius: 0,
    redaction: "erase",
    shadow: false,
    softness: 0,
    strength: 0,
    width: 56,
  },
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

  it("keeps the mark of what the pen made, which Clear all takes", () => {
    const [marked] = validAnnotations([{ ...counter("a", 1), pen: true }]);
    expect(marked.pen).toBe(true);
    const [unmarked] = validAnnotations([{ ...counter("b", 2), pen: "yes" }]);
    expect(unmarked).not.toHaveProperty("pen");
  });

  it("reads a stored text box back, tucking in a pointer it cannot read", () => {
    const box = (pointer: unknown) => ({
      ...counter("t", 1),
      shape: { kind: "text", origin: { x: 4, y: 5 }, pointer, text: "Hi" },
      style: {
        align: "center",
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
        width: 28,
      },
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

  it("reads a stored highlight back, and drops one with a band it cannot draw", () => {
    const highlight = (bands: unknown[]) => ({
      ...counter("h", 1),
      shape: {
        bands,
        end: { x: 90, y: 30 },
        kind: "highlight",
        seed: 7,
        start: { x: 10, y: 10 },
        tone: { ink: 0.9, surface: 0.1 },
      },
    });
    // A band written the wrong way round is read the right way round.
    const [read] = validAnnotations([
      highlight([{ bottom: 4, left: 90, right: 10, top: 20 }]),
    ]);
    expect(read.shape).toEqual(
      expect.objectContaining({
        bands: [{ bottom: 20, left: 10, right: 90, top: 4 }],
        tone: { ink: 0.9, surface: 0.1 },
      }),
    );
    expect(read.style.handDrawn).toBe(false);
    expect(
      validAnnotations([
        highlight([{ bottom: 4, left: "x", right: 10, top: 2 }]),
      ]),
    ).toEqual([]);
  });

  it("reads a stored magnifier back, and drops one whose loupe it cannot draw", () => {
    const magnifier = (loupe: unknown, size: unknown) => ({
      ...counter("m", 1),
      shape: {
        end: { x: 90, y: 90 },
        kind: "magnify",
        loupe,
        size,
        start: { x: 10, y: 10 },
      },
    });
    const [read] = validAnnotations([magnifier({ x: 200, y: 50 }, 160)]);
    expect(read.shape).toEqual({
      end: { x: 90, y: 90 },
      kind: "magnify",
      loupe: { x: 200, y: 50 },
      size: 160,
      start: { x: 10, y: 10 },
    });
    expect(validAnnotations([magnifier(undefined, 160)])).toEqual([]);
    expect(validAnnotations([magnifier({ x: 200, y: 50 }, -1)])).toEqual([]);
    expect(validAnnotations([magnifier({ x: 200, y: 50 }, "big")])).toEqual([]);
  });

  it("reads an older document's style as left-aligned", () => {
    const [read] = validAnnotations([
      {
        ...counter("a", 1),
        style: {
          color: "#ffcc00",
          handDrawn: false,
          head: "none",
          manual: false,
          radius: 0,
          redaction: "erase",
          softness: 0,
          strength: 0,
          width: 56,
        },
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
