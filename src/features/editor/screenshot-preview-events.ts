// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Annotation,
  AnnotationTextEdit,
  annotationTextEdit,
  validAnnotations,
} from "./annotations";
import {
  ScreenshotWorkspaceOutputSettings,
  screenshotWorkspaceItemOutput,
} from "./screenshot-output";

/**
 * The layer output a native annotation change writes into the document, and
 * which layer it is, or null where there is nothing to write. Choosing an
 * annotation reports the list the layer already holds: a selection rather
 * than an edit, which must not land in the undo history.
 */
export const screenshotAnnotationOutput = (
  workspace: ScreenshotWorkspaceOutputSettings | undefined,
  event: ScreenshotAnnotationChangeEvent,
) => {
  const item = workspace?.items[event.paneIndex];
  if (!workspace || !item) return null;
  const held = screenshotWorkspaceItemOutput(workspace, item.id);
  if (JSON.stringify(held.annotations) === JSON.stringify(event.annotations))
    return null;
  return {
    itemId: item.id,
    output: { ...held, annotations: event.annotations },
  };
};

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
    textEdit: annotationTextEdit(event.textEdit),
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
 * one edit, or as a text box is typed into, where `textEdit` says where in the
 * typing the commit falls. The native tool owns the drawing; only lists
 * arrive here.
 */
export type ScreenshotAnnotationChangeEvent = {
  annotations: Annotation[];
  paneIndex: number;
  selectedAnnotationId: string | null;
  textEdit: AnnotationTextEdit | null;
};
