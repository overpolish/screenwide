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

import { ANNOTATION_SIZES } from "../../components/shared/annotation-style/widths";

import type {
  AnnotationHead,
  AnnotationKind,
} from "../../components/shared/annotation-style/types";

/** A point in the layer source's pixel space. */
type AnnotationPoint = { x: number; y: number };

export type AnnotationStyle = {
  /** `#rrggbb` or `#rrggbbaa`, straight alpha. */
  color: string;
  head: AnnotationHead;
  /** Stroke width in output pixels. */
  width: number;
};

/** A quadratic Bézier from `start` to `end`, bent by `control`. */
type AnnotationArrow = {
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
type AnnotationCounter = {
  angle: number;
  center: AnnotationPoint;
  kind: "counter";
  value: number;
};

type AnnotationShape = AnnotationArrow | AnnotationCounter;

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
};

const annotationPoint = (value: unknown): AnnotationPoint | null => {
  const point = value as Partial<AnnotationPoint> | null | undefined;
  return typeof point?.x === "number" &&
    typeof point.y === "number" &&
    Number.isFinite(point.x) &&
    Number.isFinite(point.y)
    ? { x: point.x, y: point.y }
    : null;
};

const annotationHead = (value: unknown): AnnotationHead =>
  value === "none" || value === "both" ? value : "end";

/**
 * How long an animated annotation takes to arrive, in source milliseconds. The
 * twins of `REVEAL_DRAW_IN_MS` and `COUNTER_REVEAL_IN_MS` in
 * `src-tauri/src/editor/annotations/reveal.rs` and `counter/reveal.rs`, which
 * Rust tests hold to these lines; a reveal may shorten its phase for a short
 * clip but never lengthens it, so an annotation placed this long before the
 * playhead is always whole by the time the playhead is reached.
 */
export const ANNOTATION_DRAW_IN_MS = 1000;
const ANNOTATION_COUNTER_DRAW_IN_MS = 320;

/**
 * What one kind of annotation is and can do: how it is read from a document,
 * how long it takes to arrive, what it is called, and which controls it
 * answers to.
 *
 * One row per kind rather than a condition at each control, so a tool added
 * later states its own answers in one place and the panel, the lane and the
 * document reader all follow. The sizes come from the shared table the live
 * overlay's controls read, so a kind is offered the same sizes wherever it is
 * drawn.
 */
type AnnotationKindRow<Shape extends AnnotationShape> =
  (typeof ANNOTATION_SIZES)[AnnotationKind] & {
    drawInMs: number;
    /** Whether it is aimed by an angle of its own, rather than by its ends. */
    hasAngle: boolean;
    /** Whether it carries heads to choose between. */
    hasHead: boolean;
    /** What the timeline lane calls one, `index` being its place in the lane. */
    laneLabel: (shape: Shape, index: number) => string;
    /** The shape as stored, or null where the compositor could not place it. */
    parseShape: (value: unknown) => Shape | null;
    /** Whether it can be turned round end for end. */
    reversible: boolean;
  };

export const ANNOTATION_KINDS: {
  [Kind in AnnotationKind]: AnnotationKindRow<
    Extract<AnnotationShape, { kind: Kind }>
  >;
} = {
  arrow: {
    ...ANNOTATION_SIZES.arrow,
    drawInMs: ANNOTATION_DRAW_IN_MS,
    hasAngle: false,
    hasHead: true,
    // An arrow has no name of its own, so it is called by its place in the
    // lane.
    laneLabel: (_shape, index) => `Arrow ${String(index + 1)}`,
    parseShape: (value) => {
      const shape = (value ?? {}) as Partial<AnnotationArrow>;
      const control = annotationPoint(shape.control);
      const end = annotationPoint(shape.end);
      const start = annotationPoint(shape.start);
      return control && end && start
        ? { control, end, kind: "arrow", start }
        : null;
    },
    reversible: true,
  },
  counter: {
    ...ANNOTATION_SIZES.counter,
    drawInMs: ANNOTATION_COUNTER_DRAW_IN_MS,
    hasAngle: true,
    // A counter is a disc with a number in it, which leaves it no head to
    // choose and nothing to reverse.
    hasHead: false,
    laneLabel: (shape) => `Counter ${String(shape.value)}`,
    parseShape: (value) => {
      const shape = (value ?? {}) as Partial<AnnotationCounter>;
      const center = annotationPoint(shape.center);
      const number = shape.value;
      return center &&
        typeof shape.angle === "number" &&
        Number.isFinite(shape.angle) &&
        typeof number === "number" &&
        Number.isInteger(number) &&
        number >= 1
        ? { angle: shape.angle, center, kind: "counter", value: number }
        : null;
    },
    reversible: false,
  },
};

/** Whether `value` names a kind this build knows how to draw. */
const isAnnotationKind = (value: unknown): value is AnnotationKind =>
  typeof value === "string" && value in ANNOTATION_KINDS;

/** How long `annotation` takes to arrive: a counter grows into place far
 * quicker than an arrow draws itself. */
export const annotationDrawInMs = (annotation: Annotation) =>
  ANNOTATION_KINDS[annotation.shape.kind].drawInMs;

/** What the timeline lane calls `annotation`, `index` being its place there. */
export const annotationLaneLabel = (annotation: Annotation, index: number) => {
  const { shape } = annotation;
  // The row is looked up by the shape's own kind, so the two always agree;
  // TypeScript cannot carry that correlation through the lookup.
  const label = ANNOTATION_KINDS[shape.kind].laneLabel as (
    shape: AnnotationShape,
    index: number,
  ) => string;
  return label(shape, index);
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
 * Read a stored document's annotations, dropping anything the compositor could
 * not place. An annotation that is not wholly finite is no annotation at all
 * rather than something every kernel has to guard, and a shape this build does
 * not know belongs to a newer document than it can draw.
 */
export const validAnnotations = (value: unknown): Annotation[] => {
  if (!Array.isArray(value)) return [];
  const valid: Annotation[] = [];
  for (const entry of value as unknown[]) {
    const annotation = (entry ?? {}) as Partial<Annotation>;
    const shape = annotation.shape;
    const style = annotation.style;
    if (typeof style?.color !== "string" || typeof style.width !== "number")
      continue;
    if (!Number.isFinite(style.width)) continue;
    const dress = {
      color: style.color,
      head: annotationHead(style.head),
      width: Math.max(0, style.width),
    };
    const common = {
      aboveCamera: annotation.aboveCamera === true,
      animated: annotation.animated !== false,
      id: typeof annotation.id === "string" ? annotation.id : "",
      style: dress,
    };
    const kind = (shape as { kind?: unknown } | undefined)?.kind;
    if (!isAnnotationKind(kind)) continue;
    const parsed = ANNOTATION_KINDS[kind].parseShape(shape);
    if (parsed) valid.push({ ...common, shape: parsed });
  }
  return valid;
};

/**
 * The annotations with their counters numbered 1, 2, 3 in the order they were
 * dropped, which is the order they are stored in.
 *
 * Numbering is derived rather than kept: deleting the second of three
 * counters leaves the third reading 3 with no 2 in sight, which is not a
 * count. Renumbering on the way in means a delete, an undo and a reorder all
 * land on the same numbers without any of them knowing about counters. The
 * list is returned unchanged when nothing moved, so an unchanged document is
 * never rewritten.
 */
export const renumberedCounters = (annotations: Annotation[]): Annotation[] => {
  let value = 0;
  const renumbered = annotations.map((annotation) => {
    if (annotation.shape.kind !== "counter") return annotation;
    value += 1;
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
