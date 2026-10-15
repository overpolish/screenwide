// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Dispatch,
  RefObject,
  SetStateAction,
  useCallback,
  useMemo,
} from "react";

import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
} from "../../screenshot/screenshot-output";
import { createRecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { useRecordingKeyboardTimelineEditing } from "../../timeline/keyboard/use-recording-keyboard-timeline-editing";
import { RecordingVideoTrackId } from "../../types";
import { useEditorEditGesture } from "../../use-editor-edit-history";
import { useCaptionSelection } from "../captions/use-caption-selection";
import { useScenePreviewEditing } from "../scenes/use-scene-preview-editing";

import { overlayTargets } from "./overlay-layers";
import { RecordingCanvasTool } from "./recording-crop-toggle";
import { scenePanePicks } from "./recording-scene-picks";
import {
  recordingVideoSelectionOverlay,
  recordingVideoSelectionTargets,
} from "./recording-selection-overlay";
import { useRecordingSelectionGesture } from "./use-recording-selection-gesture";
import { useRecordingSelectionNudge } from "./use-recording-selection-nudge";

import type { RecordingSelectionGestureEvent } from "../use-recording-preview-surface";
import type { ResolvedScrubPreviewProps } from "./recording-preview-props";

/** What the preview has in hand: the video selection drawn over the panes,
 * the gestures that move and resize it, and the overlays a press opens the
 * tools of. */
export function useRecordingPreviewSelection(
  props: ResolvedScrubPreviewProps,
  {
    activeVideoTrack,
    canPreviewBakedCamera,
    canvasTool: toolInHand,
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
    onCameraOverlayChange,
    onRecordingOutputChange,
    onRecordingTimelineEditChange,
    onSelectedTrackChange,
    previewOutputDimensions,
    previewSourceDimensions,
    recordingTimelineEdit,
  } = props;
  // The scene tool picks, moves and resizes the panes the way the select tool
  // does, with the Scene panel up in place of the Selection panel. The
  // overlays and the annotations stay the select tool's own.
  const canvasTool = toolInHand === "scene" ? "select" : toolInHand;
  const laneEdit = useMemo(
    () => recordingTimelineEdit ?? createRecordingTimelineEdit(artifactId),
    [artifactId, recordingTimelineEdit],
  );
  // Shortcuts and captions are chosen one kind at a time, so Delete only
  // ever takes away one kind of thing.
  const keyboardTimeline = useRecordingKeyboardTimelineEditing({
    artifactId,
    edit: laneEdit,
    enabled: hasKeyboardData,
    onChange: onRecordingTimelineEditChange,
    onSelectionStart: () => {
      captionSelection.onClear();
      onSelectedTrackChange?.(null);
    },
    sourceDurationMs: durationMs,
  });
  const captionSelection = useCaptionSelection({
    edit: laneEdit,
    onChange: onRecordingTimelineEditChange,
    onSelectionStart: () => {
      keyboardTimeline.selection.onClear();
      onSelectedTrackChange?.(null);
    },
  });
  const clearKeyboardSelection = keyboardTimeline.selection.onClear;
  const clearCaptionSelection = captionSelection.onClear;
  const clearOverlaySelection = useCallback(() => {
    clearKeyboardSelection();
    clearCaptionSelection();
  }, [clearCaptionSelection, clearKeyboardSelection]);
  const scene = useScenePreviewEditing({
    cameraOverlay,
    cameraSource: previewSourceDimensions.camera,
    canPreviewBakedCamera,
    canvasTool,
    durationMs,
    edit: recordingTimelineEdit,
    editGesture,
    onEditChange: onRecordingTimelineEditChange,
    output: effectiveRecordingOutput.primary,
    positionMs: previewPositionMs,
  });
  const sceneArrangement = scene.arrangement;
  const isFramed = scene.isFramed;
  // The select tool moves and resizes a custom scene's boxes the way it does
  // the recording's own panes, so neither the selected pane nor the targets
  // tell the native overlay a scene fixes them.
  const framesPanes = isFramed && !(scene.isCustom && canvasTool === "select");
  const selectionCameraOverlay =
    sceneArrangement?.cameraOverlay ?? cameraOverlay;
  const selectionOutput = useMemo(
    () =>
      sceneArrangement
        ? { ...effectiveRecordingOutput, primary: sceneArrangement.output }
        : effectiveRecordingOutput,
    [effectiveRecordingOutput, sceneArrangement],
  );
  const videoSelectionOverlay = useMemo(
    () =>
      recordingVideoSelectionOverlay({
        activeVideoTrack,
        cameraOverlay: selectionCameraOverlay,
        canPreviewBakedCamera,
        canvasTool,
        effectiveRecordingOutput: selectionOutput,
        isFramed: framesPanes,
        previewSourceDimensions: {
          camera: previewSourceDimensions.camera,
          primary: previewSourceDimensions.primary,
        },
        selectedVideoTracks,
      }),
    [
      activeVideoTrack,
      canPreviewBakedCamera,
      canvasTool,
      framesPanes,
      previewSourceDimensions.camera,
      previewSourceDimensions.primary,
      selectedVideoTracks,
      selectionCameraOverlay,
      selectionOutput,
    ],
  );
  const videoSelectionTargets = useMemo(
    () =>
      recordingVideoSelectionTargets({
        cameraOverlay: selectionCameraOverlay,
        canPreviewBakedCamera,
        canvasTool,
        effectiveRecordingOutput: selectionOutput,
        isFramed: framesPanes,
        previewSourceDimensions,
        selectedVideoTracks,
      }),
    [
      canPreviewBakedCamera,
      canvasTool,
      framesPanes,
      previewSourceDimensions,
      selectedVideoTracks,
      selectionCameraOverlay,
      selectionOutput,
    ],
  );
  // While the crop tool reframes a scene's pane, its selection is the crop
  // window over the pane's unzoomed picture. Panes not in hand keep their
  // boxes, which is what the preview draws for them.
  const reframe = activeVideoTrack
    ? scene.reframeSelections?.[activeVideoTrack]
    : undefined;
  const picks = scenePanePicks({
    arrangement: sceneArrangement,
    canPreviewBakedCamera,
    freeScreen: scene.isFull && canvasTool === "select",
    isFramed,
    overlay: videoSelectionOverlay,
    reframe,
    targets: videoSelectionTargets,
  });
  const selectionOverlay = picks.overlay;
  // Last, so they are under the pointer before the panes they are drawn over.
  const selectionTargets =
    toolInHand === "select"
      ? [
          ...(picks.targets ?? []),
          ...overlayTargets({
            keyboard: hasKeyboardData && keyboardEffects.bake,
          }),
        ]
      : picks.targets;
  const selectionGesture = useRecordingSelectionGesture({
    cameraOverlay,
    canPreviewBakedCamera,
    // Inside a scene the crop tool draws the select tool's chrome, so the
    // corner radius it offers is the select tool's gesture.
    canvasTool: isFramed && canvasTool === "crop" ? "select" : canvasTool,
    editGesture,
    effectiveRecordingOutput,
    onCameraOverlayChange,
    onRecordingOutputChange,
    previewSourceDimensions,
    recenterRefreshRef,
    selectedVideoTracks,
    setCanvasResizeDraft,
  });
  const sceneGesture = scene.gesture;
  const applyGesture = (event: RecordingSelectionGestureEvent) => {
    if (!sceneGesture.applyGesture(event)) selectionGesture.applyGesture(event);
  };
  const editsBakedCameraOverlay =
    canPreviewBakedCamera && activeVideoTrack === "camera";
  const nudgeActiveTrack = useRecordingSelectionNudge({
    activeTrack: activeVideoTrack,
    applyGesture,
    cameraOverlay,
    editsBakedCamera: editsBakedCameraOverlay,
    gestureAccepted: () =>
      sceneGesture.gestureAccepted() || selectionGesture.gestureAccepted(),
    output: effectiveRecordingOutput,
    outputDimensions: previewOutputDimensions,
  });
  return {
    applyGesture,
    /** The screen and the camera at the playhead, where One video bakes it. */
    bakedLayers: canPreviewBakedCamera
      ? {
          cameraOverlay: selectionCameraOverlay,
          screen: selectionOutput.primary,
        }
      : null,
    /** The captions chosen in their lane or in the picture. */
    captionSelection,
    /** Lets go of the shortcuts and captions chosen. */
    clearOverlaySelection,
    /** Whether a scene places the panes at the playhead. */
    isFramed,
    keyboardTimeline,
    nudgeActiveTrack,
    /** The pane of the scene the crop tool is reframing, which the preview
     * draws unzoomed. */
    reframePane: reframe
      ? reframe.layerId === 0
        ? ("screen" as const)
        : ("camera" as const)
      : null,
    selectionOverlay,
    selectionTargets,
  };
}
