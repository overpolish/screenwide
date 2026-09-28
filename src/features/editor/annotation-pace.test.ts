// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { annotationDrawInMs, annotationPathMs } from "./annotation-pace";
import { Annotation, AnnotationShape } from "./annotations";
import { recordingAnnotationClipAt } from "./recording-annotations";

const FRAME = { height: 1_080, width: 1_920 };
const DIAGONAL = Math.hypot(FRAME.width, FRAME.height);

const dressed = (shape: AnnotationShape): Annotation => ({
  aboveCamera: false,
  animated: true,
  id: "a",
  shape,
  style: {
    align: "left",
    color: "#ffcc00",
    handDrawn: false,
    head: "end",
    manual: false,
    radius: 0,
    redaction: "erase",
    strength: 0,
    width: 8,
  },
});

/** A straight arrow running `share` of the picture's diagonal, scaled by
 * `scale`. */
const arrow = (share: number, scale = 1) => {
  const x = (share * DIAGONAL * scale * FRAME.width) / DIAGONAL;
  const y = (share * DIAGONAL * scale * FRAME.height) / DIAGONAL;
  return dressed({
    control: { x: x / 2, y: y / 2 },
    end: { x, y },
    kind: "arrow",
    start: { x: 0, y: 0 },
  });
};

describe("annotation pace", () => {
  it("draws a longer path for longer, but not in proportion, within its range", () => {
    expect(annotationPathMs(arrow(0.2), FRAME)).toBe(1_000);
    expect(annotationPathMs(arrow(0.45), FRAME)).toBe(1_500);
    expect(annotationPathMs(arrow(2), FRAME)).toBe(2_000);
    expect(annotationPathMs(arrow(0.02), FRAME)).toBe(600);
  });

  it("paces a path by its share of the picture, whatever the capture's scale", () => {
    const doubled = { height: FRAME.height * 2, width: FRAME.width * 2 };
    expect(annotationPathMs(arrow(0.3, 2), doubled)).toBe(
      annotationPathMs(arrow(0.3), FRAME),
    );
  });

  it("paces a shape round its outline", () => {
    const box = dressed({
      end: { x: 1_200, y: 800 },
      kind: "shape",
      seed: 1,
      start: { x: 200, y: 200 },
    });
    // 3,200 pixels round: well past the arrow's second.
    expect(annotationPathMs(box, FRAME)).toBe(2_000);
  });

  it("reaches a fresh clip back by its paced arrival", () => {
    const long = arrow(0.45);
    const clip = recordingAnnotationClipAt({
      annotation: long,
      frame: FRAME,
      sourceDurationMs: 20_000,
      sourcePositionMs: 4_000,
    });
    expect(clip.pathMs).toBe(1_500);
    expect(clip.startMs).toBe(2_500);
  });

  it("paces a text box's pointer by its reach, after the box has grown in", () => {
    const text = (reach: number) =>
      dressed({
        kind: "text",
        origin: { x: 0, y: 0 },
        pointer: { along: { x: 0, y: 1 }, reach: { x: 0, y: reach } },
        text: "Hi",
      });
    const pointed = annotationPathMs(text(12), FRAME);
    expect(pointed).toBe(560);
    expect(annotationDrawInMs(text(12), pointed)).toBe(320 + 560);
    // A pointer tucked into its box draws nothing, and the box arrives on its
    // own time.
    expect(annotationPathMs(text(0), FRAME)).toBeUndefined();
  });
});
