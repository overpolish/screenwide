// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * A stored document's annotations read back, each held to what the compositor
 * can draw.
 */

import { DEFAULT_BLUR_STRENGTH } from "../../../components/shared/annotation-style/widths";

import { ANNOTATION_KINDS, isAnnotationKind } from "./annotation-kinds";

import type { Annotation } from "./annotations";
import type {
  AnnotationAlign,
  AnnotationHead,
  AnnotationRedaction,
} from "../../../components/shared/annotation-style/types";

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

/** A share of a box's shorter side, from 0 to 50, or none where it is not a
 * number. */
const boxShare = (value: unknown) =>
  Math.min(50, Math.max(0, finiteOr(value, 0)));

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
      // Absent from a document written before spotlights could be drawn.
      blur: (style.blur as unknown) === true,
      color: style.color,
      // Absent from a document written before highlights could be drawn.
      handDrawn: (style.handDrawn as unknown) === true,
      head: annotationHead(style.head),
      manual: (style.manual as unknown) === true,
      radius: boxShare(style.radius),
      redaction: annotationRedaction(style.redaction),
      // Absent from a document written before magnifiers could be drawn.
      shadow: (style.shadow as unknown) === true,
      softness: boxShare(style.softness),
      strength: finiteOr(style.strength, DEFAULT_BLUR_STRENGTH),
      tint: (style.tint as unknown) === true,
      width: Math.max(0, style.width),
    };
    const common = {
      aboveCamera: annotation.aboveCamera === true,
      animated: annotation.animated !== false,
      id: typeof annotation.id === "string" ? annotation.id : "",
      ...(annotation.pen === true ? { pen: true as const } : {}),
      style: dress,
    };
    const kind = (shape as { kind?: unknown } | undefined)?.kind;
    if (!isAnnotationKind(kind)) continue;
    const parsed = ANNOTATION_KINDS[kind].parseShape(shape);
    if (parsed) valid.push({ ...common, shape: parsed });
  }
  return valid;
};
