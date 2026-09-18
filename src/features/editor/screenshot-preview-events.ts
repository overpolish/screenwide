// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Annotation, validAnnotations } from "./annotations";

/**
 * Reads one native annotation-change payload, dropping anything the
 * compositor could not place. `null` means the payload was not this
 * session's, or was not an annotation change at all.
 */
export const screenshotAnnotationChange = (
  payload: unknown,
  sessionId: number,
): ScreenshotAnnotationChangeEvent | null => {
  const event = (payload ?? {}) as Partial<ScreenshotAnnotationChangeEvent> & {
    sessionId?: number;
  };
  if (event.sessionId !== sessionId) return null;
  if (!Number.isInteger(event.paneIndex)) return null;
  return {
    annotations: validAnnotations(event.annotations),
    paneIndex: event.paneIndex as number,
    selectedAnnotationId:
      typeof event.selectedAnnotationId === "string"
        ? event.selectedAnnotationId
        : null,
  };
};

/**
 * Reads one native hover payload. `null` means the payload was not this
 * session's; an `annotationId` of null means the pointer rests on no
 * annotation.
 */
export const screenshotAnnotationHover = (
  payload: unknown,
  sessionId: number,
): { annotationId: string | null } | null => {
  const event = (payload ?? {}) as {
    annotationId?: unknown;
    sessionId?: number;
  };
  if (event.sessionId !== sessionId) return null;
  return {
    annotationId:
      typeof event.annotationId === "string" ? event.annotationId : null,
  };
};

export type ScreenshotSelectionGestureEvent = {
  deltaX: number;
  deltaY: number;
  edges: number;
  operation:
    | "cropMove"
    | "cropResize"
    | "frameRadius"
    | "frameResize"
    | "move"
    | "radius"
    | "resize";
  paneIndex: number;
  phase: "begin" | "update" | "end" | "cancel";
  scale: number;
};

/**
 * The layer's annotations after an arrow gesture, for the document to take as
 * one edit. The native tool owns the drawing; only the finished list arrives
 * here.
 */
export type ScreenshotAnnotationChangeEvent = {
  annotations: Annotation[];
  paneIndex: number;
  selectedAnnotationId: string | null;
};
