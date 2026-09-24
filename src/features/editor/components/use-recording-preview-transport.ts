// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Dispatch,
  RefObject,
  SetStateAction,
  useEffect,
  useMemo,
  useRef,
} from "react";

import { useRecenterInsetControls } from "../recenter-inset-channel";
import { createRecordingTimelineEdit } from "../recording-timeline-edit";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
} from "../screenshot-output";
import { EditorToolId, isAnnotationTool } from "../tool-panels/tool-registry";
import { useToolPanelFollowsTool } from "../tool-panels/use-tool-panel-follows-tool";
import { RecordingVideoTrackId } from "../types";
import { usePreviewZoom } from "../use-preview-zoom";
import { useRecordingAnnotations } from "../use-recording-annotations";
import { useRecordingPreviewPlayer } from "../use-recording-preview-player";
import { useRecordingTimelineThumbnails } from "../use-recording-timeline-thumbnails";

import { useRegisterPreviewFit } from "./preview-fit-context";
import { RecordingCanvasTool } from "./recording-crop-toggle";
import { Playhead } from "./scrub-playhead";
import { useRecordingCropPreview } from "./use-recording-crop-preview";
import { KEYBOARD_LAYER_ID } from "./use-recording-keyboard-canvas-gesture";
import { useRecordingPreviewPanes } from "./use-recording-preview-panes";
import { useRecordingPreviewSelection } from "./use-recording-preview-selection";
import { useRecordingRecenter } from "./use-recording-recenter";
import { useRecordingTimelineBlade } from "./use-recording-timeline-blade";
import { useRecordingTrimPreview } from "./use-recording-trim-preview";

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
    effectiveRecordingOutput,
    playhead,
    previewPlayingRef,
    recenterRefreshRef,
    reportZoom,
    screenCanvasRef,
    selectedStreamIndices,
    selectedVideoTracks,
    selection,
    setPreviewPositionMs,
    videoTrackOrder,
    videoTrackOrderList,
    zoomRequest,
  }: {
    activeVideoTrack: RecordingVideoTrackId | null;
    cameraCanvasRef: RefObject<HTMLCanvasElement | null>;
    canvasTool: RecordingCanvasTool;
    effectiveRecordingOutput: RecordingOutputSettings;
    playhead: Playhead;
    previewPlayingRef: RefObject<boolean>;
    recenterRefreshRef: RefObject<
      (crop: ScreenshotOutputSettings["sourceCrop"]) => void
    >;
    reportZoom: ReturnType<typeof usePreviewZoom>["reportZoom"];
    screenCanvasRef: RefObject<HTMLCanvasElement | null>;
    selectedStreamIndices: number[];
    selectedVideoTracks: Set<RecordingVideoTrackId>;
    selection: ReturnType<typeof useRecordingPreviewSelection>;
    setPreviewPositionMs: Dispatch<SetStateAction<number>>;
    videoTrackOrder: readonly RecordingVideoTrackId[];
    videoTrackOrderList: RecordingVideoTrackId[];
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
    enabledVideoTracks,
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
      previewSourceDimensions,
    });
  // The tool the annotation chrome is drawn from. A baked camera layer has no
  // annotations of its own, so selecting it puts the tool down. One value feeds
  // both the native chrome (through the layout, beside the selection it has
  // to agree with) and the editor's own delete shortcut.
  const annotationTool =
    (bakeCamera && activeVideoTrack === "camera") ||
    !isAnnotationTool(canvasTool)
      ? null
      : canvasTool;
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
    nativeLayoutHasPanes: enabledVideoTracks.length > 0,
    nativeLayoutKey: `${bakeCamera ? "baked" : "split"}|${enabledVideoTracks.join(":")}|${videoTrackOrderList.join(":")}`,
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
      if (selectedVideoTracks.has(trackId)) onSelectedTrackChange?.(trackId);
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
    trackId: activeVideoTrack
      ? bakeCamera
        ? "primary"
        : activeVideoTrack
      : null,
  });
  clearAnnotationRef.current = annotations.clearSelection;
  useToolPanelFollowsTool(
    "recording",
    recordingToolId(canvasTool),
    annotations.hasSelection,
  );
  useRegisterPreviewFit(player);
  const isPlaying = player.isPlaying;
  const getPlayerPositionMs = player.getPositionMs;
  previewPlayingRef.current = isPlaying;
  useEffect(() => {
    if (isPlaying) return;
    const frame = requestAnimationFrame(() => {
      setPreviewPositionMs(getPlayerPositionMs());
    });
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [getPlayerPositionMs, isPlaying, setPreviewPositionMs]);
  const recenter = useRecordingRecenter({
    artifactId,
    getPositionMs: player.getPositionMs,
    onOutputChange: (next) => onRecordingOutputChange?.("primary", next),
    output: effectiveRecordingOutput.primary,
    source: previewSourceDimensions.primary,
  });
  recenterRefreshRef.current = recenter.refresh;
  // The crop panel's commit and the Select panel's padding controls reach the
  // same analysis a crop drag ends with. Both read the frame under the
  // playhead, so the picture is parked before it is read.
  useRecenterInsetControls("recording", {
    begin: () => {
      player.pause();
      recenter.begin();
    },
    prepare: () => {
      player.pause();
      recenter.prepare();
    },
    refresh: recenter.refresh,
  });
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
      selectedVideoTracks,
      videoTrackOrder,
    });
  return {
    annotations,
    cameraPane: layout?.panes[1],
    clearAnnotationRef,
    layout,
    player,
    screenPane: layout?.panes[0],
    timelineBlade,
    timelineThumbnails,
    visibleCanvasRefs,
    visibleLayout,
    visiblePaneEntries,
  };
}
