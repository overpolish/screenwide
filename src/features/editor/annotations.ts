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

import { DEFAULT_BLUR_STRENGTH } from "../../components/shared/annotation-style/widths";

import { ANNOTATION_KINDS, isAnnotationKind } from "./annotation-kinds";

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
  /** `#rrggbb` or `#rrggbbaa`, straight alpha. */
  color: string;
  head: AnnotationHead;
  /** A redaction's corner radius, as a percentage of its box's shorter side
   * from 0 to 50; the other kinds carry zero. */
  radius: number;
  /** How a redaction covers what is under it; the other kinds carry the
   * default. */
  redaction: AnnotationRedaction;
  /** A blurred redaction's strength, a step from 1 to 5; the other kinds
   * carry zero. */
  strength: number;
  /** Stroke width, disc diameter, type size or pixelation block, in output
   * pixels. */
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
export type AnnotationRedact = {
  end: AnnotationPoint;
  kind: "redact";
  seed: number;
  start: AnnotationPoint;
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
  AnnotationArrow | AnnotationCounter | AnnotationRedact | AnnotationText;

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

const annotationHead = (value: unknown): AnnotationHead =>
  value === "none" || value === "both" ? value : "end";

const annotationAlign = (value: unknown): AnnotationAlign =>
  value === "center" || value === "right" ? value : "left";

const annotationRedaction = (value: unknown): AnnotationRedaction =>
  value === "blur" ||
  value === "color" ||
  value === "pixelate" ||
  value === "pixelateClassic"
    ? value
    : "erase";

/** A finite number, or `fallback` for anything else. */
const finiteOr = (value: unknown, fallback: number) =>
  typeof value === "number" && Number.isFinite(value) ? value : fallback;

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
      align: annotationAlign(style.align),
      color: style.color,
      head: annotationHead(style.head),
      radius: Math.min(50, Math.max(0, finiteOr(style.radius, 0))),
      redaction: annotationRedaction(style.redaction),
      strength: finiteOr(style.strength, DEFAULT_BLUR_STRENGTH),
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
