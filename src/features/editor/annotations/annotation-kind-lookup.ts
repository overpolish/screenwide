// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The kind table read for one value or one annotation: whether a stored kind
 * is one this build draws, and what the timeline lane calls an annotation.
 */

import { ANNOTATION_KINDS } from "./annotation-kinds";

import type { Annotation, AnnotationShape } from "./annotations";
import type { AnnotationKind } from "../../../components/shared/annotation-style/types";

/** Whether `value` names a kind this build knows how to draw. */
export const isAnnotationKind = (value: unknown): value is AnnotationKind =>
  typeof value === "string" && value in ANNOTATION_KINDS;

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
