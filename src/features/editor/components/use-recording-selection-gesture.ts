// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Dispatch, RefObject, SetStateAction, useRef } from "react";

import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
} from "../screenshot-output";
import { CameraOverlaySettings, RecordingVideoTrackId } from "../types";

import { RecordingCanvasTool } from "./recording-crop-toggle";
import {
  gesturedCameraOverlay,
  gesturedTrackOutput,
} from "./recording-selection-gesture-geometry";

import type { useEditorEditGesture } from "../use-editor-edit-history";
import type { RecordingSelectionGestureEvent } from "../use-recording-preview-surface";
import type { useRecordingKeyboardPreviewEditing } from "./use-recording-keyboard-canvas-editing";

const AUTO_FIT_MOVE_EDGE = 1 << 17;
const AUTO_FIT_COMMIT_EDGE = 1 << 18;

/**
 * Turns one native selection gesture into recording-output and camera-overlay
 * edits. The keyboard layer is offered the gesture first; everything else is a
 * move, resize, radius or crop of the pane the gesture began on.
 */
export function useRecordingSelectionGesture({
  cameraOverlay,
  canPreviewBakedCamera,
  canvasTool,
  editGesture,
  effectiveRecordingOutput,
  keyboardCanvas,
  onCameraOverlayChange,
  onRecordingOutputChange,
  previewSourceDimensions,
  recenterRefreshRef,
  selectedVideoTracks,
  setCanvasResizeDraft,
}: {
  cameraOverlay: CameraOverlaySettings;
  canPreviewBakedCamera: boolean;
  canvasTool: RecordingCanvasTool;
  editGesture: ReturnType<typeof useEditorEditGesture>;
  effectiveRecordingOutput: RecordingOutputSettings;
  keyboardCanvas: ReturnType<
    typeof useRecordingKeyboardPreviewEditing
  >["canvas"];
  previewSourceDimensions: Partial<
    Record<RecordingVideoTrackId, { height: number; width: number }>
  >;
  recenterRefreshRef: RefObject<
    (crop: ScreenshotOutputSettings["sourceCrop"]) => void
  >;
  selectedVideoTracks: ReadonlySet<RecordingVideoTrackId>;
  setCanvasResizeDraft: Dispatch<
    SetStateAction<RecordingOutputSettings | null>
  >;
  onCameraOverlayChange?: (settings: CameraOverlaySettings) => void;
  onRecordingOutputChange?: (
    trackId: RecordingVideoTrackId,
    settings: RecordingOutputSettings[RecordingVideoTrackId],
  ) => void;
}) {
  const selectionGestureRef = useRef<{
    cameraOverlaySnapshot: CameraOverlaySettings | null;
    lastDeltaX: number;
    lastDeltaY: number;
    lastScale: number;
    operation: RecordingSelectionGestureEvent["operation"];
    outputSnapshot: RecordingOutputSettings[RecordingVideoTrackId] | null;
    paneIndex: number;
    trackId: RecordingVideoTrackId;
  } | null>(null);
  const applyGesture = (event: RecordingSelectionGestureEvent) => {
    if (keyboardCanvas.applyGesture(event)) return;
    const trackId =
      event.paneIndex === 0
        ? "primary"
        : event.paneIndex === 1
          ? "camera"
          : null;
    const isFrameGesture =
      event.operation === "frameResize" || event.operation === "frameRadius";
    const isCropGesture =
      event.operation === "cropMove" || event.operation === "cropResize";
    if (event.phase === "begin") {
      if (
        !trackId ||
        (isFrameGesture
          ? canvasTool !== "canvas"
          : isCropGesture
            ? canvasTool !== "crop"
            : canvasTool !== "select") ||
        !selectedVideoTracks.has(trackId)
      )
        return;
      const editsBakedCamera =
        !isFrameGesture && canPreviewBakedCamera && trackId === "camera";
      selectionGestureRef.current = {
        cameraOverlaySnapshot: editsBakedCamera ? cameraOverlay : null,
        lastDeltaX: 0,
        lastDeltaY: 0,
        lastScale: event.scale,
        operation: event.operation,
        outputSnapshot: editsBakedCamera
          ? null
          : effectiveRecordingOutput[trackId],
        paneIndex: event.paneIndex,
        trackId,
      };
      editGesture.beginGesture();
      return;
    }
    const active = selectionGestureRef.current;
    if (
      !active ||
      event.paneIndex !== active.paneIndex ||
      event.operation !== active.operation
    )
      return;
    if (event.phase === "cancel") {
      if (active.cameraOverlaySnapshot)
        onCameraOverlayChange?.(active.cameraOverlaySnapshot);
      else if (active.outputSnapshot) {
        onRecordingOutputChange?.(active.trackId, active.outputSnapshot);
        if (
          active.operation === "frameRadius" ||
          active.operation === "frameResize"
        )
          setCanvasResizeDraft(null);
      }
      selectionGestureRef.current = null;
      requestAnimationFrame(editGesture.endGesture);
      return;
    }
    const finaliseGestureFrame = () => {
      active.lastDeltaX = event.deltaX;
      active.lastDeltaY = event.deltaY;
      active.lastScale = event.scale;
    };
    const changed =
      Math.abs(event.deltaX) > 1e-9 ||
      Math.abs(event.deltaY) > 1e-9 ||
      ((event.operation === "resize" || event.operation === "frameResize") &&
        Math.abs(event.scale - 1) > 1e-9) ||
      ((event.operation === "radius" || event.operation === "frameRadius") &&
        Math.abs(event.scale - active.lastScale) > 1e-9);
    const differsFromLastUpdate =
      Math.abs(event.deltaX - active.lastDeltaX) > 1e-9 ||
      Math.abs(event.deltaY - active.lastDeltaY) > 1e-9 ||
      ((event.operation === "resize" ||
        event.operation === "frameResize" ||
        event.operation === "radius" ||
        event.operation === "frameRadius") &&
        Math.abs(event.scale - active.lastScale) > 1e-9);
    // The final native transform can legitimately be the original snapshot
    // after snapping. Apply it when it differs from the last live frame even
    // though its delta is zero, or React will re-send stale geometry.
    const shouldApply =
      event.phase === "end" ? isCropGesture || differsFromLastUpdate : changed;
    const autoFitMove =
      event.operation === "move" && (event.edges & AUTO_FIT_MOVE_EDGE) !== 0;
    const autoFitCommit =
      event.operation === "move" && (event.edges & AUTO_FIT_COMMIT_EDGE) !== 0;
    if ((autoFitMove || autoFitCommit) && event.recordingOutput) {
      if (active.cameraOverlaySnapshot) {
        onRecordingOutputChange?.("primary", event.recordingOutput.primary);
        if (event.cameraOverlay) {
          onCameraOverlayChange?.(event.cameraOverlay);
          if (autoFitCommit) active.cameraOverlaySnapshot = event.cameraOverlay;
        }
      } else {
        onRecordingOutputChange?.(
          active.trackId,
          event.recordingOutput[active.trackId],
        );
        if (autoFitCommit)
          active.outputSnapshot = event.recordingOutput[active.trackId];
      }
      if (autoFitCommit) {
        active.lastDeltaX = 0;
        active.lastDeltaY = 0;
        active.lastScale = 1;
      } else if (event.phase === "update") finaliseGestureFrame();
      if (event.phase === "end") {
        selectionGestureRef.current = null;
        requestAnimationFrame(editGesture.endGesture);
      }
      return;
    }
    if (event.operation === "frameResize" && event.recordingOutput) {
      onRecordingOutputChange?.(
        active.trackId,
        event.recordingOutput[active.trackId],
      );
      // Resizing the baked primary frame rebases the camera overlay in the
      // same native scene. Mirror that authoritative geometry as well, or
      // React's pre-gesture percentages will move the camera at mouse-up.
      if (
        canPreviewBakedCamera &&
        active.trackId === "primary" &&
        event.cameraOverlay
      )
        onCameraOverlayChange?.(event.cameraOverlay);
      setCanvasResizeDraft(event.recordingOutput);
      if (event.phase === "update") finaliseGestureFrame();
      if (event.phase === "end") {
        selectionGestureRef.current = null;
        requestAnimationFrame(() => {
          setCanvasResizeDraft(null);
          editGesture.endGesture();
        });
      }
      return;
    }
    if (active.cameraOverlaySnapshot) {
      const next = gesturedCameraOverlay({
        event,
        primaryOutput: effectiveRecordingOutput.primary,
        start: active.cameraOverlaySnapshot,
      });
      if (shouldApply) onCameraOverlayChange?.(next);
      if (event.phase === "update") finaliseGestureFrame();
      if (event.phase === "end") {
        selectionGestureRef.current = null;
        requestAnimationFrame(editGesture.endGesture);
      }
      return;
    }
    const snapshot = active.outputSnapshot;
    if (!snapshot) return;
    const next = gesturedTrackOutput({
      event,
      previewSourceDimensions,
      snapshot,
      trackId: active.trackId,
    });
    if (!next) return;
    if (shouldApply) {
      onRecordingOutputChange?.(active.trackId, next);
      if (
        active.operation === "frameRadius" ||
        active.operation === "frameResize"
      ) {
        setCanvasResizeDraft({
          ...effectiveRecordingOutput,
          [active.trackId]: next,
        });
      }
    }
    if (event.phase === "update") finaliseGestureFrame();
    if (event.phase === "end") {
      if (isCropGesture && active.trackId === "primary")
        recenterRefreshRef.current(next.sourceCrop);
      // Mouse-up is the authoritative native transform. Apply it once more,
      // then keep the history gesture open through React's commit so a late
      // pointer update cannot become a tiny second undo entry.
      selectionGestureRef.current = null;
      requestAnimationFrame(() => {
        if (
          active.operation === "frameRadius" ||
          active.operation === "frameResize"
        )
          setCanvasResizeDraft(null);
        editGesture.endGesture();
      });
    }
  };
  return {
    applyGesture,
    gestureAccepted: () => selectionGestureRef.current !== null,
  };
}
