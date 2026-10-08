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

import type { Annotation } from "../../../bindings/Annotation";
import type { AnnotationPoint } from "../../../bindings/AnnotationPoint";
import type { AnnotationShape } from "../../../bindings/AnnotationShape";
import type { AnnotationStyle } from "../../../bindings/AnnotationStyle";
import type { HighlightBand } from "../../../bindings/HighlightBand";
import type { ImageArt } from "../../../bindings/ImageArt";
import type { ImagePlay } from "../../../bindings/ImagePlay";
import type { TextEditPhase } from "../../../bindings/TextEditPhase";
import type { TextPointer } from "../../../bindings/TextPointer";

export type {
  Annotation,
  AnnotationPoint,
  AnnotationShape,
  AnnotationStyle,
  HighlightBand,
  ImageArt,
  ImagePlay,
  TextPointer,
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

/** One line a highlight covers, in source pixels. */
export type AnnotationHighlight = {
  bands: HighlightBand[];
  end: AnnotationPoint;
  kind: "highlight";
  seed: number;
  start: AnnotationPoint;
  tone: { ink: number; surface: number };
};

/**
 * Where a native commit falls in a text box's typing: the typing began, the
 * text changed, or the typing ended. The document groups the commits of one
 * typing into a single edit.
 */
export type AnnotationTextEdit = TextEditPhase;

export const annotationTextEdit = (
  value: unknown,
): AnnotationTextEdit | null =>
  value === "begin" || value === "update" || value === "end" ? value : null;
