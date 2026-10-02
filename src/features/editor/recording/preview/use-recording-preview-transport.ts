// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Dispatch, RefObject, SetStateAction, useMemo, useRef } from "react";

import { useAnnotationDraft } from "../../annotations/annotation-draft";
import { KEYBOARD_LAYER_ID } from "../../keyboard-effect/use-recording-keyboard-canvas-gesture";
import { useRegisterPreviewFit } from "../../preview/preview-fit-context";
import { usePreviewZoom } from "../../preview/use-preview-zoom";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
} from "../../screenshot/screenshot-output";
import { createRecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { useRecordingTimelineBlade } from "../../timeline/editing/use-recording-timeline-blade";
import { useRecordingTrimPreview } from "../../timeline/editing/use-recording-trim-preview";
import { Playhead } from "../../timeline/scrub-playhead";
import { useRecordingTimelineThumbnails } from "../../timeline/tracks/use-recording-timeline-thumbnails";
import {
  EditorToolId,
  isAnnotationTool,
} from "../../tool-panels/tool-registry";
import { useToolPanelFollowsTool } from "../../tool-panels/use-tool-panel-follows-tool";
import { RecordingVideoTrackId } from "../../types";
import { useRecordingAnnotations } from "../annotations/use-recording-annotations";
import { useRecordingScenes } from "../scenes/use-recording-scenes";
import { useSceneReframeDisplay } from "../scenes/use-scene-reframe-display";
import { useRecordingPreviewPlayer } from "../use-recording-preview-player";

import { RecordingCanvasTool } from "./recording-crop-toggle";
import { usePausedPreviewPosition } from "./use-paused-preview-position";
import { useRecordingCropPreview } from "./use-recording-crop-preview";
import { useRecordingPreviewPanes } from "./use-recording-preview-panes";
import { useRecordingPreviewSelection } from "./use-recording-preview-selection";
import { useRecordingRecenter } from "./use-recording-recenter";

import type { ResolvedScrubPreviewProps } from "./recording-preview-props";

/** The toolbar's own name for a tool, in the registry's vocabulary. */
const recordingToolId = (tool: RecordingCanvasTool): EditorToolId | null =>
  tool === "canvas" ? "frame" : tool;

/** Playback and the timeline under it: the native player, the annotations on
 * the clip, the recentre analysis, and the pane layout the picture is drawn
 * into. */
export function useRecordingPreviewTransport(
  props: ResolvedScrubPreviewProps,
  {
    activeVideoTrack,
    cameraCanvasRef,
    canvasTool,
    composedVideoTracks,
    effectiveRecordingOutput,
    playhead,
    previewPlayingRef,
    previewPositionMs,
    recenterRefreshRef,
    reportZoom,
    screenCanvasRef,
    selectedStreamIndices,
    selection,
    setPreviewPositionMs,
    zoomRequest,
  }: {
    activeVideoTrack: RecordingVideoTrackId | null;
    cameraCanvasRef: RefObject<HTMLCanvasElement | null>;
    canvasTool: RecordingCanvasTool;
    /** The video tracks the workspace draws, which leave out a camera saved
     * as a separate file. */
    composedVideoTracks: Set<RecordingVideoTrackId>;
    effectiveRecordingOutput: RecordingOutputSettings;
    playhead: Playhead;
    previewPlayingRef: RefObject<boolean>;
    previewPositionMs: number;
    recenterRefreshRef: RefObject<
      (crop: ScreenshotOutputSettings["sourceCrop"]) => void
    >;
    reportZoom: ReturnType<typeof usePreviewZoom>["reportZoom"];
    screenCanvasRef: RefObject<HTMLCanvasElement | null>;
    selectedStreamIndices: number[];
    selection: ReturnType<typeof useRecordingPreviewSelection>;
    setPreviewPositionMs: Dispatch<SetStateAction<number>>;
    zoomRequest: ReturnType<typeof usePreviewZoom>["zoomRequest"];
  },
) {
  const {
    artifactId,
    audioTrackVolumes,
    bakeCamera,
    cameraOverlay,
    cursorEffects,
    durationMs,
    isSaving,
    isSheetOpen,
    keyboardEffects,
    onRecordingOutputChange,
    onRecordingTimelineEditChange,
    onSelectedTrackChange,
    previewLayout,
    previewSourceDimensions,
    recordingTimelineEdit,
  } = props;
  const { keyboardCanvas, keyboardTimeline } = selection;
  const totalDurationRef = useRef(durationMs);
  const trimPreview = useRecordingTrimPreview({
    edit: recordingTimelineEdit,
    playhead,
    totalDurationRef,
  });
  // Storybook renders a fixed layout with no backend session behind it; every
  // other caller lets the native workspace editor own the layout.
  const nativeEditorOwnsLayout = previewLayout === undefined;
  // A running save covers the viewport with the progress overlay, whose Cancel
  // button is a DOM control - and the native interaction view is inserted
  // above the webview, so it would swallow that click and pan the workspace
  // instead. Suspending takes the interaction view and the native chrome off
  // screen for the duration of the save (every path that clears `isSaving`:
  // success, failure and cancel) and changes nothing else: the editor stays
  // enabled, the panes keep rendering natively, and the workspace zoom and pan
  // are exactly where the user left them when the overlay goes away. An export
  // or confirm sheet over the editor covers the workspace in the same way, and
  // is suspended for the same reason.
  const isEditorSuspended = nativeEditorOwnsLayout && (isSaving || isSheetOpen);
  const { previewCameraOverlay, previewRecordingOutput } =
    useRecordingCropPreview({
      activeVideoTrack,
      bakeCamera,
      cameraOverlay,
      canvasTool,
      effectiveRecordingOutput,
      isFramed: selection.isFramed,
      previewSourceDimensions,
    });
  // The tool the annotation chrome is drawn from. A baked camera layer has no
  // annotations of its own, and a camera saved as a separate file is not drawn
  // at all, so selecting either puts the tool down. One value feeds both the
  // native chrome (through the layout, beside the selection it has to agree
  // with) and the editor's own delete shortcut.
  const annotationTool =
    (activeVideoTrack === "camera" &&
      (bakeCamera || !composedVideoTracks.has("camera"))) ||
    !isAnnotationTool(canvasTool)
      ? null
      : canvasTool;
  const composedTrackList = [...composedVideoTracks];
  const clearAnnotationRef = useRef(() => {});
  const player = useRecordingPreviewPlayer({
    annotationTool,
    artifactId,
    audioTrackVolumes,
    bakeCamera,
    cameraCanvasRef,
    cameraOverlay: previewCameraOverlay,
    cursorEffects,
    enabledStreamIndices: selectedStreamIndices,
    isEditorSuspended,
    isEnabled: previewLayout === undefined,
    keyboardEffects,
    nativeEditorOwnsLayout,
    nativeLayoutHasPanes: composedTrackList.length > 0,
    nativeLayoutKey: `${bakeCamera ? "baked" : "split"}|${composedTrackList.join(":")}`,
    onPosition: (positionMs) => {
      trimPreview.onPosition(positionMs);
      if (!previewPlayingRef.current) setPreviewPositionMs(positionMs);
    },
    onSelectionChange: (paneIndex) => {
      if (paneIndex === null) return;
      clearAnnotationRef.current();
      if (paneIndex === KEYBOARD_LAYER_ID) {
        keyboardCanvas.selectVisible();
        onSelectedTrackChange?.(null);
        return;
      }
      keyboardTimeline.selection.onClear();
      const trackId = paneIndex === 0 ? "primary" : "camera";
      if (composedVideoTracks.has(trackId)) onSelectedTrackChange?.(trackId);
    },
    onSelectionGesture: selection.applyGesture,
    onZoomChange: reportZoom,
    recordingOutput: previewRecordingOutput,
    screenCanvasRef,
    selection: selection.selectionOverlay,
    selectionTargets: selection.selectionTargets,
    sourceDurationMs: durationMs,
    timelineEdit: recordingTimelineEdit,
    zoomRequest,
  });
  const annotationEdit = useMemo(
    () => recordingTimelineEdit ?? createRecordingTimelineEdit(artifactId),
    [recordingTimelineEdit, artifactId],
  );
  const annotations = useRecordingAnnotations({
    edit: annotationEdit,
    frames: previewSourceDimensions,
    getPositionMs: player.getPositionMs,
    onEdit: (next) => onRecordingTimelineEditChange?.(next),
    onSelectTrack: (track) => {
      if (player.isPlaying) player.pause();
      keyboardTimeline.selection.onClear();
      timelineBlade.blade.clearRangeSelection();
      timelineBlade.blade.selectSegment(null);
      onSelectedTrackChange?.(track);
    },
    sessionId: player.sessionId,
    sourceDurationMs: durationMs,
    tool: annotationTool,
    trackId:
      activeVideoTrack && composedVideoTracks.has(activeVideoTrack)
        ? bakeCamera
          ? "primary"
          : activeVideoTrack
        : null,
  });
  clearAnnotationRef.current = annotations.clearSelection;
  const scenes = useRecordingScenes({
    cameraOverlay,
    edit: annotationEdit,
    frames: previewSourceDimensions,
    getPositionMs: player.getPositionMs,
    isBaked: bakeCamera,
    isPlaying: player.isPlaying,
    onEdit: (next) => onRecordingTimelineEditChange?.(next),
    output: effectiveRecordingOutput.primary,
    playhead,
    positionMs: previewPositionMs,
    selectsCamera: activeVideoTrack === "camera",
    sessionId: player.sessionId,
    sourceDurationMs: durationMs,
  });
  useSceneReframeDisplay(player.sessionId, selection.reframePane);
  useToolPanelFollowsTool(
    "recording",
    recordingToolId(canvasTool),
    annotations.hasSelection,
  );
  useAnnotationDraft("recording", recordingToolId(canvasTool));
  useRegisterPreviewFit(player);
  previewPlayingRef.current = player.isPlaying;
  usePausedPreviewPosition({
    getPositionMs: player.getPositionMs,
    isPlaying: player.isPlaying,
    setPreviewPositionMs,
  });
  recenterRefreshRef.current = useRecordingRecenter({
    artifactId,
    getPositionMs: player.getPositionMs,
    onOutputChange: (next) => onRecordingOutputChange?.("primary", next),
    output: effectiveRecordingOutput.primary,
    pause: player.pause,
    source: previewSourceDimensions.primary,
  }).refresh;
  const timelineThumbnails = useRecordingTimelineThumbnails({
    artifactId,
    isEnabled: previewLayout === undefined,
  });
  const totalDurationMs = player.durationMs || durationMs;
  totalDurationRef.current = totalDurationMs;
  const layout = player.layout ?? previewLayout ?? null;
  const timelineBlade = useRecordingTimelineBlade({
    artifactId,
    edit: recordingTimelineEdit,
    framesPerSecond: player.framesPerSecond,
    getPositionMs: player.getPositionMs,
    onChange: onRecordingTimelineEditChange,
    onTrimPreviewRestore: trimPreview.restore,
    onTrimPreviewStart: trimPreview.start,
    ownsDelete: !annotations.canDelete,
    playhead,
    seekPlayer: player.seek,
    shortcutsEnabled: Boolean(layout),
    totalDurationMs,
  });
  const { visibleCanvasRefs, visibleLayout, visiblePaneEntries } =
    useRecordingPreviewPanes({
      cameraCanvasRef,
      layout,
      screenCanvasRef,
      selectedVideoTracks: composedVideoTracks,
    });
  return {
    annotations,
    cameraPane: layout?.panes[1],
    clearAnnotationRef,
    layout,
    player,
    scenes,
    screenPane: layout?.panes[0],
    timelineBlade,
    timelineThumbnails,
    visibleCanvasRefs,
    visibleLayout,
    visiblePaneEntries,
  };
}
