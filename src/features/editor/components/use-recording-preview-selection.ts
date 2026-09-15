// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Dispatch, RefObject, SetStateAction, useMemo } from "react";

import { usePublishKeyboardShortcut } from "../keyboard-shortcut-channel";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
} from "../screenshot-output";
import { RecordingVideoTrackId } from "../types";
import { useEditorEditGesture } from "../use-editor-edit-history";

import { RecordingCanvasTool } from "./recording-crop-toggle";
import {
  recordingVideoSelectionOverlay,
  recordingVideoSelectionTargets,
} from "./recording-selection-overlay";
import { useRecordingKeyboardPreviewEditing } from "./use-recording-keyboard-canvas-editing";
import { useRecordingSelectionGesture } from "./use-recording-selection-gesture";
import { useRecordingSelectionNudge } from "./use-recording-selection-nudge";

import type { ResolvedScrubPreviewProps } from "./recording-preview-props";

/** A normalized coordinate as the panel's percent field shows it, held to two
 * decimals so a redrawn frame is not published as a new value. */
const roundedPercent = (value: number) => Math.round(value * 10000) / 100;

/** What the preview has in hand: the shortcut layer, the video selection drawn
 * over the panes, and the gestures that move and resize either of them. */
export function useRecordingPreviewSelection(
  props: ResolvedScrubPreviewProps,
  {
    activeVideoTrack,
    canPreviewBakedCamera,
    canvasTool,
    editGesture,
    effectiveRecordingOutput,
    previewPositionMs,
    recenterRefreshRef,
    selectedVideoTracks,
    setCanvasResizeDraft,
  }: {
    activeVideoTrack: RecordingVideoTrackId | null;
    canPreviewBakedCamera: boolean;
    canvasTool: RecordingCanvasTool;
    editGesture: ReturnType<typeof useEditorEditGesture>;
    effectiveRecordingOutput: RecordingOutputSettings;
    previewPositionMs: number;
    recenterRefreshRef: RefObject<
      (crop: ScreenshotOutputSettings["sourceCrop"]) => void
    >;
    selectedVideoTracks: Set<RecordingVideoTrackId>;
    setCanvasResizeDraft: Dispatch<
      SetStateAction<RecordingOutputSettings | null>
    >;
  },
) {
  const {
    artifactId,
    cameraOverlay,
    durationMs,
    hasKeyboardData,
    keyboardEffects,
    keyboardMaximumWidthUnits,
    onCameraOverlayChange,
    onKeyboardEffectsChange,
    onRecordingOutputChange,
    onRecordingTimelineEditChange,
    onSelectedTrackChange,
    previewOutputDimensions,
    previewSourceDimensions,
    recordingTimelineEdit,
  } = props;
  const keyboardPreview = useRecordingKeyboardPreviewEditing({
    artifactId,
    canvasTool,
    durationMs,
    edit: recordingTimelineEdit,
    enabled: hasKeyboardData,
    keyboardEffects,
    maximumWidthUnits: keyboardMaximumWidthUnits,
    onChange: onRecordingTimelineEditChange,
    onKeyboardEffectsChange,
    onSelectionStart: () => onSelectedTrackChange?.(null),
    output: effectiveRecordingOutput.primary,
    positionMs: previewPositionMs,
  });
  const keyboardCanvas = keyboardPreview.canvas;
  const keyboardTimeline = keyboardPreview.timeline;
  const visibleKeyboardFragment = keyboardPreview.visibleFragment;
  const videoSelectionOverlay = useMemo(
    () =>
      recordingVideoSelectionOverlay({
        activeVideoTrack,
        cameraOverlay,
        canPreviewBakedCamera,
        canvasTool,
        effectiveRecordingOutput,
        previewSourceDimensions: {
          camera: previewSourceDimensions.camera,
          primary: previewSourceDimensions.primary,
        },
        selectedVideoTracks,
      }),
    [
      activeVideoTrack,
      canPreviewBakedCamera,
      cameraOverlay,
      canvasTool,
      effectiveRecordingOutput,
      previewSourceDimensions.camera,
      previewSourceDimensions.primary,
      selectedVideoTracks,
    ],
  );
  const videoSelectionTargets = useMemo(
    () =>
      recordingVideoSelectionTargets({
        cameraOverlay,
        canPreviewBakedCamera,
        canvasTool,
        effectiveRecordingOutput,
        previewSourceDimensions,
        selectedVideoTracks,
      }),
    [
      canPreviewBakedCamera,
      cameraOverlay,
      canvasTool,
      effectiveRecordingOutput,
      previewSourceDimensions,
      selectedVideoTracks,
    ],
  );
  const keyboardSelection = keyboardPreview.selection;
  // The shortcut is drawn for whichever fragment is on screen, but it is in
  // hand only once it has been picked out.
  const hasKeyboardSelection = Boolean(
    keyboardSelection &&
    keyboardTimeline.selection.ids.has(
      visibleKeyboardFragment?.fragmentId ?? "",
    ),
  );
  const selectionOverlay = hasKeyboardSelection
    ? keyboardSelection
    : videoSelectionOverlay;
  const selectionTargets = keyboardSelection
    ? [...(videoSelectionTargets ?? []), keyboardSelection]
    : videoSelectionTargets;
  // The shortcut in hand, for the Selection panel in the panel window.
  const keyboardGeometry = keyboardPreview.geometry;
  usePublishKeyboardShortcut(
    "recording",
    hasKeyboardSelection && keyboardGeometry
      ? {
          kind: "shortcut",
          label: "Shortcut",
          maximumSizePercent: keyboardGeometry.maximumSizePercent,
          minimumSizePercent: keyboardGeometry.minimumSizePercent,
          positionXPercent: roundedPercent(keyboardGeometry.center.x),
          positionYPercent: roundedPercent(keyboardGeometry.center.y),
          sizePercent: keyboardGeometry.sizePercent,
        }
      : null,
    keyboardCanvas,
  );
  const selectionGesture = useRecordingSelectionGesture({
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
  });
  const editsBakedCameraOverlay =
    canPreviewBakedCamera && activeVideoTrack === "camera";
  const nudgeActiveTrack = useRecordingSelectionNudge({
    activeTrack: activeVideoTrack,
    applyGesture: selectionGesture.applyGesture,
    cameraOverlay,
    editsBakedCamera: editsBakedCameraOverlay,
    gestureAccepted: selectionGesture.gestureAccepted,
    output: effectiveRecordingOutput,
    outputDimensions: previewOutputDimensions,
  });
  return {
    applyGesture: selectionGesture.applyGesture,
    keyboardCanvas,
    keyboardTimeline,
    nudgeActiveTrack,
    selectionOverlay,
    selectionTargets,
  };
}
