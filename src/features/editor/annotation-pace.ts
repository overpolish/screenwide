// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * How long an annotation's path takes to draw in on a recording, from how
 * long the path is.
 *
 * Drawn in a fixed time, a long path races and a short one crawls. Drawn at a
 * constant speed, a box round half the picture would take ages. So the time
 * grows with the square root of the length, as a hand's does: a typical
 * arrow keeps its second, a long box slows but does not crawl, and both are
 * held to a range where the drawing still reads as drawing. Lengths are
 * shares of the picture's diagonal, so a box looks the same pace on a 1x and a
 * 2x capture.
 *
 * What travels is the whole of an arrow, a highlight's lines laid end to end
 * and a shape's outline. A text box grows into place first, on a counter's
 * fixed time, and only its pointer travels, so the pointer is paced by its
 * own length in ems of the box's type. A counter and a redaction grow into
 * place and have no path.
 *
 * The pace is worked out here and kept on the clip as `pathMs`, which the
 * native preview and both exports read; `reveal.rs` holds the unpaced times a
 * clip without one falls back to.
 */

import { ANNOTATION_DRAW_IN_MS, ANNOTATION_KINDS } from "./annotation-kinds";

import type { Annotation } from "./annotations";
import type { RecordingAnnotationClip } from "./recording-annotations";

/** A picture's size in its own source pixels. */
export type AnnotationFrame = { height: number; width: number };

/** A pace: the time taken over the reference length, and the least and most
 * any length may take. */
type Pace = { base: number; max: number; min: number };

/** A path the unpaced second draws runs a fifth of the picture's diagonal,
 * about a typical arrow's. */
const REFERENCE_SHARE = 0.2;
const PATH_PACE: Pace = { base: ANNOTATION_DRAW_IN_MS, max: 2_000, min: 600 };

/** How long a text box's pointer takes to draw out where it names no pace,
 * and the reach in ems it takes that long over. The twin of `POINTER_IN_MS`
 * in `src-tauri/src/editor/annotations/text/reveal.rs`. */
const POINTER_DRAW_IN_MS = 280;
const POINTER_REFERENCE_EMS = 3;
const POINTER_PACE: Pace = { base: POINTER_DRAW_IN_MS, max: 900, min: 200 };

/** The time `pace` takes over `ratio` times its reference length: grown by
 * the ratio's square root and held to the pace's range. */
const paced = (ratio: number, { base, max, min }: Pace) =>
  Math.round(Math.min(max, Math.max(min, base * Math.sqrt(ratio))));

/** How far along a quadratic Bézier from `a` to `c` bent by `b`. */
const curveLength = (
  a: { x: number; y: number },
  b: { x: number; y: number },
  c: { x: number; y: number },
) => {
  const at = (t: number) => {
    const u = 1 - t;
    return {
      x: u * u * a.x + 2 * u * t * b.x + t * t * c.x,
      y: u * u * a.y + 2 * u * t * b.y + t * t * c.y,
    };
  };
  let length = 0;
  let previous = a;
  for (let step = 1; step <= 32; step++) {
    const next = at(step / 32);
    length += Math.hypot(next.x - previous.x, next.y - previous.y);
    previous = next;
  }
  return length;
};

/** How far an annotation's path runs, in source pixels, or null for one that
 * travels no path of its own there. */
const pathLength = ({ shape, style }: Annotation) => {
  switch (shape.kind) {
    case "arrow":
      return curveLength(shape.start, shape.control, shape.end);
    case "highlight":
      return shape.bands.reduce(
        (sum, band) => sum + Math.max(0, band.right - band.left),
        0,
      );
    case "shape": {
      const across = Math.abs(shape.end.x - shape.start.x);
      const down = Math.abs(shape.end.y - shape.start.y);
      const rounding =
        (Math.min(across, down) * Math.min(50, Math.max(0, style.radius))) /
        100;
      return 2 * (across + down) - 8 * rounding + 2 * Math.PI * rounding;
    }
    case "counter":
    case "redact":
    case "text":
      return null;
  }
};

/**
 * How long `annotation`'s path takes to draw in, on a picture `frame` in
 * size, or undefined for one with no path, which arrives on its kind's own
 * time.
 */
export const annotationPathMs = (
  annotation: Annotation,
  frame: AnnotationFrame | undefined,
): number | undefined => {
  const shape = annotation.shape;
  if (shape.kind === "text") {
    const reach = Math.hypot(shape.pointer.reach.x, shape.pointer.reach.y);
    return reach > 0
      ? paced(reach / POINTER_REFERENCE_EMS, POINTER_PACE)
      : undefined;
  }
  const length = pathLength(annotation);
  const diagonal = frame ? Math.hypot(frame.width, frame.height) : 0;
  if (length === null || !(diagonal > 0) || !Number.isFinite(length))
    return undefined;
  return paced(length / diagonal / REFERENCE_SHARE, PATH_PACE);
};

/**
 * How long `annotation` takes to arrive at its clip's pace `pathMs`: an
 * arrow, a highlight or a shape over its whole path, a text box over its
 * box's growing and then its pointer. Without a pace, each kind's own time. A
 * redaction that does not animate is whole from its clip's first frame, so
 * its clip starts where it is placed rather than reaching back.
 */
export const annotationDrawInMs = (annotation: Annotation, pathMs?: number) => {
  const { kind } = annotation.shape;
  if (kind === "redact" && !annotation.animated) return 0;
  const unpaced = ANNOTATION_KINDS[kind].drawInMs;
  if (pathMs === undefined) return unpaced;
  return kind === "text" ? unpaced - POINTER_DRAW_IN_MS + pathMs : pathMs;
};

/** `clip` paced by its annotation as it now stands. */
export const pacedClip = (
  clip: RecordingAnnotationClip,
  frame: AnnotationFrame | undefined,
): RecordingAnnotationClip => {
  const pathMs = annotationPathMs(clip.annotation, frame);
  if (pathMs === clip.pathMs) return clip;
  const { pathMs: _stale, ...rest } = clip;
  return pathMs === undefined ? rest : { ...rest, pathMs };
};
