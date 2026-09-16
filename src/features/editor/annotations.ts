// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * Drawn marks carried by an editor layer.
 *
 * Points are in the source's own pixel space, the same space the crop is
 * expressed in, so a mark stays glued to the picture while the frame is
 * moved, resized or re-cropped. The compositor draws them, which is what
 * keeps the editor's preview and the exported PNG the same image. This is the
 * twin of `src-tauri/src/editor/annotations/model.rs`.
 */

import { AnnotationHead } from "../../components/shared/annotation-style/types";

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
type AnnotationShape = {
  control: AnnotationPoint;
  end: AnnotationPoint;
  kind: "arrow";
  start: AnnotationPoint;
};

export type Annotation = {
  /**
   * Whether the mark is drawn over the camera bubble rather than under it.
   * Screenshots have no bubble; the recording compositor will honour this.
   */
  aboveCamera: boolean;
  /**
   * Whether a timed mark draws itself in at the start of its clip and undraws
   * at the end. Stills have no clip to animate over and draw whole.
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
 * How long an animated mark takes to draw itself in, in source milliseconds.
 * The twin of `REVEAL_DRAW_IN_MS` in
 * `src-tauri/src/editor/annotations/reveal.rs`, which a Rust test holds to
 * this line; the reveal may shorten the phase for a short clip but never
 * lengthens it, so a mark placed this long before the playhead is always
 * finished drawing by the time the playhead is reached.
 */
export const ANNOTATION_DRAW_IN_MS = 1000;

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
 * Read a stored document's marks, dropping anything the compositor could not
 * place. A mark that is not wholly finite is no mark at all rather than
 * something every kernel has to guard, and a shape this build does not know
 * belongs to a newer document than it can draw.
 */
export const validAnnotations = (value: unknown): Annotation[] => {
  if (!Array.isArray(value)) return [];
  const valid: Annotation[] = [];
  for (const entry of value as unknown[]) {
    const annotation = (entry ?? {}) as Partial<Annotation>;
    const shape = annotation.shape;
    const style = annotation.style;
    if (shape?.kind !== "arrow") continue;
    if (typeof style?.color !== "string" || typeof style.width !== "number")
      continue;
    const control = annotationPoint(shape.control);
    const end = annotationPoint(shape.end);
    const start = annotationPoint(shape.start);
    if (!control || !end || !start || !Number.isFinite(style.width)) continue;
    valid.push({
      aboveCamera: annotation.aboveCamera === true,
      animated: annotation.animated !== false,
      id: typeof annotation.id === "string" ? annotation.id : "",
      shape: { control, end, kind: "arrow", start },
      style: {
        color: style.color,
        head: annotationHead(style.head),
        width: Math.max(0, style.width),
      },
    });
  }
  return valid;
};
