// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { CaptureSource } from "../layer-measure";

import { validAnnotations } from "./annotation-documents";
import { mappedAnnotationShape } from "./annotation-mapping";
import { Annotation, AnnotationPoint } from "./annotations";

// Annotations copied in either editor, held natively so another editor
// window can paste them (`annotation_clipboard.rs`). They travel with the
// capture they were drawn on, since their points are its pixels: a paste
// keeps their size in points and their place as a share of the picture.

/** How far a paste steps from annotations it would land on, and how far a
 * duplicate stands from its original, in points. */
const PASTE_STEP_POINTS = 16;

/** One copied annotation, and how long its clip ran where it was copied
 * from a recording. */
type CopiedAnnotation = { annotation: Annotation; lengthMs?: number };

export type AnnotationClipboard = {
  capture: CaptureSource;
  items: CopiedAnnotation[];
};

/** What the clipboard asks of an editor's annotation choice. */
export type AnnotationChoice = {
  deleteSelected: () => void;
  onSelectedIdsChange: (ids: readonly string[]) => void;
  selectedIds: ReadonlySet<string>;
};

export const copyAnnotations = (clipboard: AnnotationClipboard) =>
  invoke("copy_annotations", { copied: clipboard });

const finitePositive = (value: unknown): value is number =>
  typeof value === "number" && Number.isFinite(value) && value > 0;

/** The clipboard as copied, or null where it holds nothing usable. */
export const parsedAnnotationClipboard = (
  value: unknown,
): AnnotationClipboard | null => {
  if (typeof value !== "object" || value === null) return null;
  if (!("capture" in value) || !("items" in value)) return null;
  const { capture, items } = value;
  if (
    typeof capture !== "object" ||
    capture === null ||
    !("width" in capture) ||
    !("height" in capture) ||
    !("scaleFactor" in capture) ||
    !finitePositive(capture.width) ||
    !finitePositive(capture.height) ||
    !finitePositive(capture.scaleFactor) ||
    !Array.isArray(items)
  )
    return null;
  const copied = items.flatMap((item: unknown): CopiedAnnotation[] => {
    if (typeof item !== "object" || item === null || !("annotation" in item))
      return [];
    const valid = validAnnotations([item.annotation]);
    if (valid.length === 0) return [];
    const annotation = valid[0];
    const lengthMs =
      "lengthMs" in item && finitePositive(item.lengthMs)
        ? item.lengthMs
        : undefined;
    return [lengthMs === undefined ? { annotation } : { annotation, lengthMs }];
  });
  if (copied.length === 0) return null;
  return {
    capture: {
      height: capture.height,
      scaleFactor: capture.scaleFactor,
      width: capture.width,
    },
    items: copied,
  };
};

/** The annotations copied last, unless something else was copied since. */
export const readCopiedAnnotations = async () =>
  parsedAnnotationClipboard(await invoke<unknown>("copied_annotations"));

/** Every point `shape` is placed by. */
const shapePoints = (annotation: Annotation) => {
  const points: AnnotationPoint[] = [];
  mappedAnnotationShape(annotation.shape, (point) => {
    points.push(point);
    return point;
  });
  return points;
};

const boundsMiddle = (points: AnnotationPoint[]) => {
  const xs = points.map((point) => point.x);
  const ys = points.map((point) => point.y);
  return {
    x: (Math.min(...xs) + Math.max(...xs)) / 2,
    y: (Math.min(...ys) + Math.max(...ys)) / 2,
  };
};

/** `annotations` carried `offset` source pixels. */
export const movedAnnotations = (
  annotations: Annotation[],
  offset: { x: number; y: number },
) =>
  annotations.map((annotation) => ({
    ...annotation,
    shape: mappedAnnotationShape(annotation.shape, (point) => ({
      x: point.x + offset.x,
      y: point.y + offset.y,
    })),
  }));

/** The clipboard's annotations as drawn on `target`: their size kept in
 * points, the middle of what they cover at the same share of the picture. */
const onCapture = (
  clipboard: AnnotationClipboard,
  target: CaptureSource,
): Annotation[] => {
  const annotations = clipboard.items.map((item) => item.annotation);
  const { capture } = clipboard;
  const points = annotations.flatMap(shapePoints);
  if (points.length === 0) return annotations;
  const middle = boundsMiddle(points);
  const landed = {
    x: (middle.x / capture.width) * target.width,
    y: (middle.y / capture.height) * target.height,
  };
  const scale = target.scaleFactor / capture.scaleFactor;
  return annotations.map((annotation) => ({
    ...annotation,
    shape: mappedAnnotationShape(annotation.shape, (point) => ({
      x: landed.x + (point.x - middle.x) * scale,
      y: landed.y + (point.y - middle.y) * scale,
    })),
  }));
};

const sameShape = (a: Annotation, b: Annotation) =>
  JSON.stringify(a.shape) === JSON.stringify(b.shape);

/**
 * The clipboard's annotations pasted onto `target`, which already holds
 * `existing`: each given an id of its own, and where one would land exactly
 * on an annotation already there, as a paste back onto the picture it was
 * copied from does, the whole paste stepped `PASTE_STEP_POINTS` down and
 * right until none does, so pasting again fans the copies out.
 */
export const pastedAnnotations = (
  clipboard: AnnotationClipboard,
  target: CaptureSource,
  existing: readonly Annotation[],
): Annotation[] => {
  const placed = onCapture(clipboard, target);
  const step = PASTE_STEP_POINTS * target.scaleFactor;
  let pasted = placed;
  for (let steps = 1; steps <= 50; steps += 1) {
    if (!pasted.some((one) => existing.some((other) => sameShape(one, other))))
      break;
    pasted = movedAnnotations(placed, { x: step * steps, y: step * steps });
  }
  return pasted.map((annotation) => ({
    ...annotation,
    id: crypto.randomUUID(),
  }));
};
