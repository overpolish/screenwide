// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Dispatch,
  RefObject,
  SetStateAction,
  useCallback,
  useMemo,
} from "react";

import { openingCropChrome } from "../../preview/opening-crop-chrome";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
} from "../../screenshot/screenshot-output";
import { createRecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { useRecordingKeyboardTimelineEditing } from "../../timeline/keyboard/use-recording-keyboard-timeline-editing";
import { isInRecordingTimelineGap } from "../../timeline/recording-strip/recording-timeline-gaps";
import { useCropAspectStore } from "../../tools/crop/crop-aspect-store";
import { RecordingVideoTrackId } from "../../types";
import { useEditorEditGesture } from "../../use-editor-edit-history";
import { useCaptionSelection } from "../captions/use-caption-selection";
import { useScenePreviewEditing } from "../scenes/use-scene-preview-editing";

import { useFollowedOverlay } from "./face-path";
import { overlayTargets } from "./overlay-layers";
import { RecordingCanvasTool } from "./recording-canvas-tool";
import { scenePanePicks } from "./recording-scene-picks";
import { withCropShape } from "./recording-selection";
import {
  recordingVideoSelectionOverlay,
  recordingVideoSelectionTargets,
} from "./recording-selection-overlay";
import { useRecordingCropTurn } from "./use-recording-crop-turn";
import { useRecordingSelectionGesture } from "./use-recording-selection-gesture";
import { useRecordingSelectionNudge } from "./use-recording-selection-nudge";

import type {
  RecordingPreviewSelection,
  RecordingSelectionGestureEvent,
} from "../use-recording-preview-surface";
import type { ResolvedScrubPreviewProps } from "./recording-preview-props";

/** What the preview has in hand: the video selection drawn over the panes,
 * the gestures that move and resize it, and the overlays a press opens the
 * tools of. */
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
      (crop: ScreenshotOutputSettings["sourceCrop"]) => Promise<void>
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
    canPreviewBakedCamera,
    canvasTool,
    captures: previewSourceDimensions,
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
  const turn = useRecordingCropTurn({
    activeVideoTrack,
    arrangement: sceneArrangement,
    bakesCamera: canPreviewBakedCamera,
    canvasTool,
    isFramed,
    screenRotation: effectiveRecordingOutput.primary.rotation,
  });
  const { chromeTool, chromeTrack } = turn;
  const followedCameraOverlay = useFollowedOverlay(
    cameraOverlay,
    previewSourceDimensions.camera,
    previewPositionMs,
  );
  const selectionCameraOverlay =
    sceneArrangement?.cameraOverlay ?? followedCameraOverlay;
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
        activeVideoTrack: chromeTrack,
        cameraOverlay: selectionCameraOverlay,
        canPreviewBakedCamera,
        canvasTool: chromeTool,
        effectiveRecordingOutput: selectionOutput,
        isFramed: framesPanes,
        previewSourceDimensions: {
          camera: previewSourceDimensions.camera,
          primary: previewSourceDimensions.primary,
        },
        selectedVideoTracks,
      }),
    [
      chromeTrack,
      canPreviewBakedCamera,
      chromeTool,
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
        canvasTool: chromeTool,
        effectiveRecordingOutput: selectionOutput,
        isFramed: framesPanes,
        previewSourceDimensions,
        selectedVideoTracks,
      }),
    [
      canPreviewBakedCamera,
      chromeTool,
      framesPanes,
      previewSourceDimensions,
      selectedVideoTracks,
      selectionCameraOverlay,
      selectionOutput,
    ],
  );
  // While the crop tool is in hand or the crop has yet to close on a scene's
  // pane, its selection is the crop window; other panes keep their boxes.
  const cropTrack = chromeTrack ?? "primary";
  const reframe =
    chromeTool === "crop" ? scene.reframeSelections?.[cropTrack] : undefined;
  const picks = scenePanePicks({
    arrangement: turn.arrangement,
    canPreviewBakedCamera,
    freeCamera: scene.freesCamera && canvasTool === "select",
    freeScreen: scene.freesScreen && canvasTool === "select",
    isFramed,
    isInGap: isInRecordingTimelineGap(laneEdit, durationMs, previewPositionMs),
    overlay: videoSelectionOverlay,
    reframe,
    screenRotation: turn.screenRotation,
    targets: videoSelectionTargets,
  });
  // A crop window keeps the shape chosen for its crop as its handles pull.
  const cropAspects = useCropAspectStore((state) => state.aspects);
  const sceneId = sceneArrangement?.clip.id ?? null;
  const shaped = (selection: RecordingPreviewSelection) =>
    withCropShape(selection, cropAspects, sceneId);
  const selectionOverlay =
    picks.overlay && shaped(openingCropChrome(picks.overlay, turn.cropTurn));
  // Last, so they are under the pointer before the panes they are drawn over.
  const selectionTargets =
    canvasTool === "select"
      ? [
          ...(picks.targets ?? []),
          ...overlayTargets({
            keyboard: hasKeyboardData && keyboardEffects.bake,
          }),
        ]
      : picks.targets?.map(shaped);
  const selectionGesture = useRecordingSelectionGesture({
    cameraOverlay,
    canPreviewBakedCamera,
    canvasTool,
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
    /** The crop tool's ease, and the track it eases, kept while it closes. */
    cropTurn: turn.cropTurn,
    cropTurnTrack: turn.turnedTrack,
    followedCameraOverlay,
    /** Whether a scene places the panes at the playhead. */
    isFramed,
    keyboardTimeline,
    nudgeActiveTrack,
    /** The pane of the scene the crop opens out, which the preview draws. */
    reframePane: turn.openPane,
    selectionOverlay,
    selectionTargets,
  };
}
