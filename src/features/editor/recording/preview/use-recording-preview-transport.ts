// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Dispatch, RefObject, SetStateAction, useMemo, useRef } from "react";

import {
  recordingBetween,
  useTweenedOutput,
} from "../../preview/use-tweened-output";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
} from "../../screenshot/screenshot-output";
import { createRecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { useRecordingTimelineBlade } from "../../timeline/editing/use-recording-timeline-blade";
import { useRecordingTrimPreview } from "../../timeline/editing/use-recording-trim-preview";
import { Playhead } from "../../timeline/scrub-playhead";
import { useRecordingTimelineThumbnails } from "../../timeline/tracks/use-recording-timeline-thumbnails";
import { isAnnotationTool } from "../../tools/tool-registry";
import { RecordingVideoTrackId } from "../../types";
import { useRecordingAnnotations } from "../annotations/use-recording-annotations";
import { useRecordingScenes } from "../scenes/use-recording-scenes";
import { useSceneReframeDisplay } from "../scenes/use-scene-reframe-display";
import { useRecordingPreviewPlayer } from "../use-recording-preview-player";

import { isOverlayLayer, OverlayLayerId } from "./overlay-layers";
import { RecordingCanvasTool } from "./recording-canvas-tool";
import { usePausedPreviewPosition } from "./use-paused-preview-position";
import { useRecordingCropPreview } from "./use-recording-crop-preview";
import { useRecordingPreviewPanes } from "./use-recording-preview-panes";
import { useRecordingPreviewSelection } from "./use-recording-preview-selection";
import { useRecordingRecenter } from "./use-recording-recenter";

import type { ResolvedScrubPreviewProps } from "./recording-preview-props";

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
    isSelectHeld,
    playhead,
    pressOverlayRef,
    previewPlayingRef,
    previewPositionMs,
    recenterRefreshRef,
    screenCanvasRef,
    selectedStreamIndices,
    selectedVideoTracks,
    selection,
    setPreviewPositionMs,
  }: {
    activeVideoTrack: RecordingVideoTrackId | null;
    cameraCanvasRef: RefObject<HTMLCanvasElement | null>;
    canvasTool: RecordingCanvasTool;
    /** The video tracks the workspace draws, which leave out a camera saved
     * as a separate file. */
    composedVideoTracks: Set<RecordingVideoTrackId>;
    effectiveRecordingOutput: RecordingOutputSettings;
    isSelectHeld: boolean;
    playhead: Playhead;
    /** What a press on an overlay does, set by the lanes, which know it. */
    pressOverlayRef: RefObject<(layerId: OverlayLayerId) => void>;
    previewPlayingRef: RefObject<boolean>;
    previewPositionMs: number;
    recenterRefreshRef: RefObject<
      (crop: ScreenshotOutputSettings["sourceCrop"]) => Promise<void>
    >;
    screenCanvasRef: RefObject<HTMLCanvasElement | null>;
    selectedStreamIndices: number[];
    /** The video tracks turned on, which leave out one the user turned off. */
    selectedVideoTracks: Set<RecordingVideoTrackId>;
    selection: ReturnType<typeof useRecordingPreviewSelection>;
    setPreviewPositionMs: Dispatch<SetStateAction<number>>;
  },
) {
  const {
    artifactId,
    audioTrackVolumes,
    bakeCamera,
    cameraOverlay,
    cursorEffects,
    durationMs,
    isExportOpen,
    keyboardEffects,
    onRecordingOutputChange,
    onRecordingTimelineEditChange,
    onSelectedTrackChange,
    previewLayout,
    previewSourceDimensions,
    recordingTimelineEdit,
  } = props;
  const { clearOverlaySelection } = selection;
  const totalDurationRef = useRef(durationMs);
  const trimPreview = useRecordingTrimPreview({
    edit: recordingTimelineEdit,
    playhead,
    totalDurationRef,
  });
  // Storybook renders a fixed layout with no backend session behind it; every
  // other caller lets the native workspace editor own the layout.
  const nativeEditorOwnsLayout = previewLayout === undefined;
  // While the Export island stands over the editor nothing on the picture is
  // edited: the picture is the export's preview. Suspending takes the native
  // interaction view and chrome off screen and changes nothing else: the
  // panes keep rendering and playing natively, and the workspace zoom and pan
  // are exactly where the user left them when the island goes away.
  const isEditorSuspended = nativeEditorOwnsLayout && isExportOpen;
  const { previewCameraOverlay, previewRecordingOutput } =
    useRecordingCropPreview({
      activeVideoTrack,
      bakeCamera,
      cameraOverlay,
      canvasTool,
      effectiveRecordingOutput,
      previewSourceDimensions,
      selection,
    });
  // A setting chosen on an island is drawn as the picture moving to it, the
  // camera drawn into it moving with the screen.
  const drawn = useTweenedOutput(
    useMemo(
      () => ({
        cameraOverlay: previewCameraOverlay,
        recordingOutput: previewRecordingOutput,
      }),
      [previewCameraOverlay, previewRecordingOutput],
    ),
    { part: "picture", workspace: "recording" },
    recordingBetween,
  );
  // The tool the annotation chrome is drawn from. A camera saved as a
  // separate file is not drawn at all, so selecting it puts the tool down; a
  // camera drawn into the video carries annotations of its own, as the
  // screen does. One value feeds both the native chrome (through the layout,
  // beside the selection it has to agree with) and the editor's own delete
  // shortcut.
  const annotationTool =
    (activeVideoTrack === "camera" && !composedVideoTracks.has("camera")) ||
    !isAnnotationTool(canvasTool)
      ? null
      : canvasTool;
  const composedTrackList = [...composedVideoTracks];
  const clearAnnotationRef = useRef(() => {});
  const player = useRecordingPreviewPlayer({
    artifactId,
    audioTrackVolumes,
    bakeCamera,
    cameraCanvasRef,
    cameraOverlay: drawn.cameraOverlay,
    chrome: { annotationTool, selectHeld: isSelectHeld },
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
      // Nothing chosen with Select in hand lets go of everything at once:
      // the annotations, the track, and with it the Screen or Camera mode
      // it opened. The engine reports it as one change, so the editor never
      // draws the track chosen without its annotations in between.
      if (paneIndex === null) {
        if (canvasTool !== "select") return;
        clearAnnotationRef.current();
        if (activeVideoTrack !== null) onSelectedTrackChange?.(null);
        return;
      }
      // An overlay takes its size and place from its tool, so a press on it
      // chooses it and opens that tool rather than taking up a pane.
      if (isOverlayLayer(paneIndex)) {
        pressOverlayRef.current(paneIndex);
        return;
      }
      clearAnnotationRef.current();
      clearOverlaySelection();
      const trackId = paneIndex === 0 ? "primary" : "camera";
      if (composedVideoTracks.has(trackId)) onSelectedTrackChange?.(trackId);
    },
    onSelectionGesture: selection.applyGesture,
    recordingOutput: drawn.recordingOutput,
    screenCanvasRef,
    selection: selection.selectionOverlay,
    selectionTargets: selection.selectionTargets,
    sourceDurationMs: durationMs,
    timelineEdit: recordingTimelineEdit,
  });
  const annotationEdit = useMemo(
    () => recordingTimelineEdit ?? createRecordingTimelineEdit(artifactId),
    [recordingTimelineEdit, artifactId],
  );
  const scenes = useRecordingScenes({
    artifactId,
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
  const annotations = useRecordingAnnotations({
    bakedLayers: selection.bakedLayers,
    edit: annotationEdit,
    frames: previewSourceDimensions,
    getPositionMs: player.getPositionMs,
    // Deleting the annotations chosen with Select leaves nothing chosen:
    // their track goes with them rather than opening its mode.
    onChoiceDeleted: () => {
      if (canvasTool === "select") onSelectedTrackChange?.(null);
    },
    onEdit: (next) => onRecordingTimelineEditChange?.(next),
    onSelectTrack: (track) => {
      if (player.isPlaying) player.pause();
      clearOverlaySelection();
      timelineBlade.blade.clearRangeSelection();
      timelineBlade.blade.selectSegment(null);
      onSelectedTrackChange?.(track);
    },
    playhead,
    sceneId: scenes.current?.id ?? null,
    screenCaptureScale: props.screenCaptureScale ?? 1,
    sessionId: player.sessionId,
    shownTracks: selectedVideoTracks,
    sourceDurationMs: durationMs,
    tool: annotationTool,
    trackId:
      activeVideoTrack && composedVideoTracks.has(activeVideoTrack)
        ? activeVideoTrack
        : null,
  });
  clearAnnotationRef.current = annotations.clearSelection;
  useSceneReframeDisplay(player.sessionId, selection);
  previewPlayingRef.current = player.isPlaying;
  usePausedPreviewPosition(player, previewPositionMs, setPreviewPositionMs);
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
    ownsDelete: !annotations.canDelete && scenes.selection.ids.size === 0,
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
