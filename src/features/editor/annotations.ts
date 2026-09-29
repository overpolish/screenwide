// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * Drawn annotations carried by an editor layer.
 *
 * Points are in the source's own pixel space, the same space the crop is
 * expressed in, so an annotation stays glued to the picture while the frame is
 * moved, resized or re-cropped. The compositor draws them, which is what
 * keeps the editor's preview and the exported PNG the same image. This is the
 * twin of `src-tauri/src/editor/annotations/model.rs`.
 */

import type {
  AnnotationAlign,
  AnnotationHead,
  AnnotationRedaction,
} from "../../components/shared/annotation-style/types";

/** A point in the layer source's pixel space. */
export type AnnotationPoint = { x: number; y: number };

export type AnnotationStyle = {
  /** How a text box lines up its lines; the other kinds carry the default. */
  align: AnnotationAlign;
  /** Whether a spotlight also blurs what lies outside it; the other kinds
   * carry `false`. */
  blur: boolean;
  /** `#rrggbb` or `#rrggbbaa`, straight alpha. */
  color: string;
  /** Whether a highlight is drawn as a marker stroke by hand rather than as a
   * clean band, or a shape's outline as a pen stroke that misses its own
   * start; the other kinds carry `false`. */
  handDrawn: boolean;
  head: AnnotationHead;
  /** Whether a highlight is laid by hand over the box its drag spans, in
   * strokes `width` tall, rather than fitted to the text under it; the other
   * kinds carry `false`. */
  manual: boolean;
  /** A redaction's, a shape's or a spotlight's corner radius, as a
   * percentage of its box's shorter side from 0 to 50; the other kinds carry
   * zero. */
  radius: number;
  /** How a redaction covers what is under it; the other kinds carry the
   * default. */
  redaction: AnnotationRedaction;
  /** How far a spotlight's edge fades from lit to dim, as a percentage of its
   * box's shorter side from 0 to 50; the other kinds carry zero. */
  softness: number;
  /** A blurred redaction's strength, a step from 1 to 5; the other kinds
   * carry zero. */
  strength: number;
  /** Stroke width, disc diameter, type size, pixelation block, a shape's pen
   * or a highlight's marker, in points of the capture. */
  width: number;
};

/** A quadratic Bézier from `start` to `end`, bent by `control`. */
export type AnnotationArrow = {
  control: AnnotationPoint;
  end: AnnotationPoint;
  kind: "arrow";
  start: AnnotationPoint;
};

/**
 * A numbered disc with a pin's curved tail. `value` is the annotation's place
 * in the document's counter order, which the editor keeps contiguous, and
 * `angle` is where the tail points, in radians clockwise from east in the
 * source's own pixel space - so a fresh counter's zero points right.
 */
export type AnnotationCounter = {
  angle: number;
  center: AnnotationPoint;
  kind: "counter";
  value: number;
};

/**
 * Lines of type in a solid box. `origin` is the box's top-left corner and
 * `pointer` the pointer drawn out of it, held against the box. The box's size
 * follows from the text and the type size.
 */
export type AnnotationText = {
  kind: "text";
  origin: AnnotationPoint;
  pointer: TextPointer;
  text: string;
};

/**
 * A box that hides what is under it. `start` is its top-left corner and `end`
 * its bottom-right, in source pixels; `seed` generates a pixelated box's
 * blocks.
 */
type AnnotationRedact = {
  end: AnnotationPoint;
  kind: "redact";
  seed: number;
  start: AnnotationPoint;
};

/**
 * An outline round a box, drawn with a round pen. `start` is its top-left
 * corner and `end` its bottom-right, in source pixels; the style's radius
 * rounds its corners, so a square rounded all the way is a circle. `seed` is
 * its hand-drawn stroke's wobble. The twin of `AnnotationShape::Shape` in
 * `src-tauri/src/editor/annotations/shape.rs`.
 */
type AnnotationOutline = {
  end: AnnotationPoint;
  kind: "shape";
  seed: number;
  start: AnnotationPoint;
};

/**
 * A box left bright while everything around it dims. `start` is its top-left
 * corner and `end` its bottom-right, in source pixels; the style rounds its
 * corners, fades its edge and says whether what is outside it is blurred
 * too. The twin of `AnnotationShape::Spotlight` in
 * `src-tauri/src/editor/annotations/shape.rs`.
 */
type AnnotationSpotlight = {
  end: AnnotationPoint;
  kind: "spotlight";
  start: AnnotationPoint;
};

/**
 * A line drawn freehand: the points the hand passed through, in source
 * pixels, thinned as it was drawn. `smooth` fits the line loosely enough that
 * a wobbly curve comes out clean. The twin of `AnnotationShape::Draw` in
 * `src-tauri/src/editor/annotations/shape.rs`.
 */
type AnnotationDraw = {
  kind: "draw";
  points: AnnotationPoint[];
  smooth: boolean;
};

/** One line a highlight covers, in source pixels. */
export type HighlightBand = {
  bottom: number;
  left: number;
  right: number;
  top: number;
};

/**
 * A marker over lines of text. `start` and `end` are where the selection was
 * pressed and let go, in source pixels; `bands` what it covers, one per line in
 * reading order; `tone` how bright the page it was read from is, and its ink,
 * 0 to 1; `seed` its hand-drawn stroke's wobble. The twin of
 * `AnnotationShape::Highlight` in `src-tauri/src/editor/annotations/shape.rs`.
 */
export type AnnotationHighlight = {
  bands: HighlightBand[];
  end: AnnotationPoint;
  kind: "highlight";
  seed: number;
  start: AnnotationPoint;
  tone: { ink: number; surface: number };
};

/**
 * A text box's pointer, held against its box so it keeps its place however
 * the box is moved, retyped or resized. `along` is where the tip sits in each
 * axis as a share of the box's half size from its centre, -1 to 1; `reach` is
 * how far past that edge it goes, in ems of the box's type. One that reaches
 * nowhere is tucked in and not drawn. The twin of `TextPointer` in
 * `src-tauri/src/editor/annotations/text/model.rs`.
 */
export type TextPointer = {
  along: AnnotationPoint;
  reach: AnnotationPoint;
};

export type AnnotationShape =
  | AnnotationArrow
  | AnnotationCounter
  | AnnotationDraw
  | AnnotationHighlight
  | AnnotationOutline
  | AnnotationRedact
  | AnnotationSpotlight
  | AnnotationText;

export type Annotation = {
  /**
   * Whether the annotation is drawn over the camera bubble rather than under
   * it. Screenshots have no bubble; the recording compositor will honour this.
   */
  aboveCamera: boolean;
  /**
   * Whether a timed annotation draws itself in at the start of its clip and
   * undraws at the end. Stills have no clip to animate over and draw whole.
   */
  animated: boolean;
  id: string;
  shape: AnnotationShape;
  style: AnnotationStyle;
  /**
   * Set where the draw tool made the annotation: a stroke, or what a held
   * stroke was taken for. Clear all takes these and leaves the rest.
   */
  pen?: true;
};

/**
 * Which arrow a delete acts on: the one the halo is showing, and otherwise
 * the one the handles are on.
 *
 * The halo is under the pointer, so it is what the hand is pointing at; the
 * selection is what it last pointed at. An id that names no arrow on this
 * layer - a stale hover from a layer that has moved on - is no target at all.
 */
export const annotationDeleteTarget = (
  annotations: Annotation[],
  hoveredId: string | null,
  selectedId: string | null,
) => {
  const present = (id: string | null) =>
    id !== null && annotations.some((annotation) => annotation.id === id)
      ? id
      : null;
  return present(hoveredId) ?? present(selectedId);
};

/**
 * Where a native commit falls in a text box's typing: the typing began, the
 * text changed, or the typing ended. The document groups the commits of one
 * typing into a single edit. The twin of `TextEditPhase` in
 * `src-tauri/src/editor/annotations/text/edit.rs`.
 */
export type AnnotationTextEdit = "begin" | "update" | "end";

export const annotationTextEdit = (
  value: unknown,
): AnnotationTextEdit | null =>
  value === "begin" || value === "update" || value === "end" ? value : null;

/**
 * The annotations with their counters numbered 1, 2, 3 in the order of the
 * numbers they already carry, the one earlier in the list first on a tie.
 *
 * A counter keeps the number it was given: moving it, or changing which
 * annotations it is drawn over, never renumbers it. Only the gaps close, so
 * deleting the second of three counters leaves 1 and 2 rather than 1 and 3,
 * and a delete, an undo and a reorder all land on the same numbers without
 * any of them knowing about counters. The list is returned unchanged when
 * nothing moved, so an unchanged document is never rewritten.
 */
export const renumberedCounters = (annotations: Annotation[]): Annotation[] => {
  const order = annotations
    .flatMap((annotation, index) =>
      annotation.shape.kind === "counter"
        ? [{ index, value: annotation.shape.value }]
        : [],
    )
    .sort((a, b) => a.value - b.value || a.index - b.index);
  const values = new Map(order.map(({ index }, place) => [index, place + 1]));
  const renumbered = annotations.map((annotation, index) => {
    const value = values.get(index);
    if (annotation.shape.kind !== "counter" || value === undefined)
      return annotation;
    return annotation.shape.value === value
      ? annotation
      : { ...annotation, shape: { ...annotation.shape, value } };
  });
  return renumbered.every(
    (annotation, index) => annotation === annotations[index],
  )
    ? annotations
    : renumbered;
};
