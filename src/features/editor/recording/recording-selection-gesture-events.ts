// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { RecordingSelectionGestureEvent } from "./use-recording-preview-surface";

/** A selection gesture as the native preview reports it. */
export type SelectionGesturePayload = Omit<
  RecordingSelectionGestureEvent,
  "operation"
> & {
  operation: number;
  sessionId: number;
};

const OPERATIONS: RecordingSelectionGestureEvent["operation"][] = [
  "move",
  "resize",
  "frameResize",
  "cropMove",
  "cropResize",
  "cropDraw",
  "rotate",
];

/** `payload` as the editor takes it, or null where it is malformed. */
export const gestureFromPayload = (
  payload: SelectionGesturePayload,
): RecordingSelectionGestureEvent | null => {
  if (
    !Number.isInteger(payload.operation) ||
    payload.operation < 0 ||
    payload.operation >= OPERATIONS.length ||
    !Number.isFinite(payload.deltaX) ||
    !Number.isFinite(payload.deltaY) ||
    !Number.isInteger(payload.edges) ||
    !Number.isInteger(payload.paneIndex) ||
    !Number.isFinite(payload.scale) ||
    !["begin", "update", "end", "cancel"].includes(payload.phase)
  )
    return null;
  return {
    cameraOverlay: payload.cameraOverlay,
    deltaX: payload.deltaX,
    deltaY: payload.deltaY,
    edges: payload.edges,
    operation: OPERATIONS[payload.operation],
    paneIndex: payload.paneIndex,
    phase: payload.phase,
    recordingOutput: payload.recordingOutput,
    scale: payload.scale,
  };
};

/**
 * Hands `deliver` a gesture's events at most one update a frame. Each update
 * carries the gesture's whole travel from the press, so only the newest
 * matters; taken one by one, each re-rendering the editor, they queued up
 * behind the pointer and the picture went on catching up for a second or
 * more after the press was let go. A phase change takes the waiting update
 * first, so the gesture's events keep their order.
 */
export const frameCoalescedGestures = (
  deliver: (event: RecordingSelectionGestureEvent) => void,
) => {
  let waiting: RecordingSelectionGestureEvent | null = null;
  let frame = 0;
  const takeWaiting = () => {
    cancelAnimationFrame(frame);
    frame = 0;
    const next = waiting;
    waiting = null;
    if (next) deliver(next);
  };
  return {
    dispose: () => {
      cancelAnimationFrame(frame);
      waiting = null;
    },
    take: (event: RecordingSelectionGestureEvent) => {
      if (event.phase === "update") {
        waiting = event;
        if (!frame) frame = requestAnimationFrame(takeWaiting);
        return;
      }
      takeWaiting();
      deliver(event);
    },
  };
};
