// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Dispatch, RefObject, SetStateAction, useMemo } from "react";

import { useRecordingKeyboardPreviewEditing } from "../../keyboard-effect/use-recording-keyboard-canvas-editing";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
} from "../../screenshot/screenshot-output";
import { usePublishKeyboardShortcut } from "../../shortcuts/keyboard-shortcut-channel";
import { RecordingVideoTrackId } from "../../types";
import { useEditorEditGesture } from "../../use-editor-edit-history";
import { useScenePreviewEditing } from "../scenes/use-scene-preview-editing";

import { RecordingCanvasTool } from "./recording-crop-toggle";
import {
  recordingVideoSelectionOverlay,
  recordingVideoSelectionTargets,
} from "./recording-selection-overlay";
import { useRecordingSelectionGesture } from "./use-recording-selection-gesture";
import { useRecordingSelectionNudge } from "./use-recording-selection-nudge";

import type {
  RecordingPreviewSelection,
  RecordingSelectionGestureEvent,
} from "../use-recording-preview-surface";
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
  // The scene tool picks, moves and resizes the panes the way the select tool
  // does, with the Scene panel up in place of the Selection panel. The
  // shortcut layer and the annotations stay the select tool's own.
  const canvasTool = toolInHand === "scene" ? "select" : toolInHand;
  const keyboardPreview = useRecordingKeyboardPreviewEditing({
    artifactId,
    canvasTool: toolInHand,
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
  const keyboardSelection = keyboardPreview.selection;
  // The shortcut is drawn for whichever fragment is on screen, but it is in
  // hand only once it has been picked out.
  const hasKeyboardSelection = Boolean(
    keyboardSelection &&
    keyboardTimeline.selection.ids.has(
      visibleKeyboardFragment?.fragmentId ?? "",
    ),
  );
  // While the crop tool reframes a scene's pane, its selection is the crop
  // window over the pane's unzoomed picture. Panes not in hand keep their
  // boxes, which is what the preview draws for them.
  const reframe = activeVideoTrack
    ? scene.reframeSelections?.[activeVideoTrack]
    : undefined;
  // A custom scene's box moves and resizes like any layer, but the canvas it
  // sits on is the scene's to size, so dragging it never grows the canvas.
  const placedFreely = (selection: RecordingPreviewSelection) =>
    isFramed &&
    !selection.framed &&
    !selection.cropMode &&
    (selection.layerId ?? selection.paneIndex) <= 1
      ? { ...selection, inScene: true }
      : selection;
  // A pane the scene hides is neither drawn nor picked.
  const opacity = sceneArrangement?.opacity;
  const isHidden = (selection: RecordingPreviewSelection) => {
    const layer = selection.layerId ?? selection.paneIndex;
    return (
      opacity !== undefined &&
      ((layer === 0 && opacity.screen === 0) ||
        (layer === 1 && canPreviewBakedCamera && opacity.camera === 0))
    );
  };
  const paneOverlay = reframe ?? videoSelectionOverlay;
  const selectionOverlay = hasKeyboardSelection
    ? keyboardSelection
    : paneOverlay && !isHidden(paneOverlay)
      ? placedFreely(paneOverlay)
      : null;
  const sceneTargets = videoSelectionTargets
    ?.filter((target) => !isHidden(target))
    .map((target) =>
      reframe && target.layerId === reframe.layerId
        ? reframe
        : placedFreely(target),
    );
  const selectionTargets = keyboardSelection
    ? [...(sceneTargets ?? []), keyboardSelection]
    : sceneTargets;
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
    // Inside a scene the crop tool draws the select tool's chrome, so the
    // corner radius it offers is the select tool's gesture.
    canvasTool: isFramed && canvasTool === "crop" ? "select" : canvasTool,
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
    /** Whether a scene places the panes at the playhead. */
    isFramed,
    keyboardCanvas,
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
