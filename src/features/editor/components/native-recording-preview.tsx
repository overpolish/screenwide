// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
  defaultRecordingOutput,
} from "../screenshot-output";
import { useCanvasTool } from "../tool-panels/use-canvas-tool";
import { RecordingVideoTrackId } from "../types";
import { useCopyRecordingFrame } from "../use-copy-recording-frame";
import { useEditorEditGesture } from "../use-editor-edit-history";
import { usePreviewZoom } from "../use-preview-zoom";

import { PreviewZoomField } from "./preview-readouts";
import { RecordingCanvasTool } from "./recording-crop-toggle";
import { resolveScrubPreviewProps } from "./recording-preview-props";
import { RecordingPreviewStage } from "./recording-preview-stage";
import { RecordingPreviewTimelineBand } from "./recording-preview-timeline-band";
import { createPlayhead } from "./scrub-playhead";
import { useRecordingCanvasContextMenu } from "./use-recording-canvas-context-menu";
import { useRecordingPreviewCanvasTool } from "./use-recording-preview-canvas-tool";
import { useRecordingPreviewSelection } from "./use-recording-preview-selection";
import { useRecordingPreviewShortcuts } from "./use-recording-preview-shortcuts";
import { useRecordingPreviewTrackOrder } from "./use-recording-preview-track-order";
import { useRecordingPreviewTracks } from "./use-recording-preview-tracks";
import { useRecordingPreviewTransport } from "./use-recording-preview-transport";
import { useRecordingToolbar } from "./use-recording-toolbar";
import { useRecordingTrackSelection } from "./use-recording-track-selection";

import type { ScrubPreviewProps } from "./scrub-preview";

/** Editor playback whose decode, audio output and timeline are all owned by Rust. */
export function NativeRecordingPreview(rawProps: ScrubPreviewProps) {
  const props = resolveScrubPreviewProps(rawProps);
  const {
    artifactId,
    bakeCamera,
    cameraOverlay,
    cursorEffects,
    hasCursorData,
    hasKeyboardData,
    keyboardEffects,
    onEnabledTracksChange,
    onEnabledVideoTracksChange,
    onSelectedTrackChange,
    onVideoTrackOrderChange,
    previewOutputDimensions,
    recordingOutput,
    selectedTrack,
  } = props;
  const screenCanvasRef = useRef<HTMLCanvasElement>(null);
  const cameraCanvasRef = useRef<HTMLCanvasElement>(null);
  const recenterRefreshRef = useRef<
    (crop: ScreenshotOutputSettings["sourceCrop"]) => void
  >(() => undefined);
  const editGesture = useEditorEditGesture();
  const [playhead] = useState(createPlayhead);
  const { reportZoom, requestZoom, zoomPercent, zoomRequest } =
    usePreviewZoom();
  const [canvasTool, setCanvasTool] = useCanvasTool<
    Exclude<RecordingCanvasTool, null>
  >("recording", "select");
  // A canvas resize runs at pointer rate; committing every move to the export
  // window's state re-renders the inspector, lanes and timeline and starves
  // the native pane's layout loop. The gesture renders from this draft and
  // commits once on release, exactly like the screenshot editor.
  const [canvasResizeDraft, setCanvasResizeDraft] =
    useState<RecordingOutputSettings | null>(null);
  const [previewPositionMs, setPreviewPositionMs] = useState(0);
  const previewPlayingRef = useRef(false);
  const activeVideoTrack =
    selectedTrack === "primary" || selectedTrack === "camera"
      ? selectedTrack
      : null;
  const activeRecordingOutput =
    (canvasTool === "canvas" ? canvasResizeDraft : null) ?? recordingOutput;
  const effectiveRecordingOutput =
    activeRecordingOutput ??
    defaultRecordingOutput({
      camera: previewOutputDimensions?.camera,
      primary: previewOutputDimensions?.primary ?? { height: 64, width: 64 },
    });
  const {
    audioVolumeByStream,
    enabledTracks,
    selectedStreamIndices,
    selectedVideoTracks,
    videoTrackOrder,
    videoTrackOrderList,
  } = useRecordingPreviewTracks(props, effectiveRecordingOutput);
  const canPreviewBakedCamera =
    bakeCamera &&
    selectedVideoTracks.has("primary") &&
    selectedVideoTracks.has("camera");
  const selection = useRecordingPreviewSelection(props, {
    activeVideoTrack,
    canPreviewBakedCamera,
    canvasTool,
    editGesture,
    effectiveRecordingOutput,
    previewPositionMs,
    recenterRefreshRef,
    selectedVideoTracks,
    setCanvasResizeDraft,
  });
  const transport = useRecordingPreviewTransport(props, {
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
  });
  const {
    annotations,
    clearAnnotationRef,
    layout,
    player,
    visiblePaneEntries,
  } = transport;
  const isPlaying = player.isPlaying;
  const canEditActiveTrack =
    activeVideoTrack !== null && selectedVideoTracks.has(activeVideoTrack);
  const canResizeActiveTrack =
    canEditActiveTrack && (!bakeCamera || canPreviewBakedCamera);
  const canMoveActiveVideoTrack =
    activeVideoTrack !== null && selectedVideoTracks.has(activeVideoTrack);
  const hasVisiblePanes = visiblePaneEntries.length > 0;
  const {
    moveActiveVideoTrackBackward,
    moveActiveVideoTrackForward,
    moveVideoTrack,
  } = useRecordingPreviewTrackOrder({
    activeVideoTrack,
    onVideoTrackOrderChange,
    videoTrackOrder,
  });
  useRecordingCanvasContextMenu({
    canvasTool,
    moveVideoTrack,
    onSelectedTrackChange,
    videoTrackOrderList,
    visiblePaneEntries,
  });
  const { changeCanvasTool, isCropping, leaveCropTool, toggleTool } =
    useRecordingPreviewCanvasTool({
      activeVideoTrack,
      bakeCamera,
      canvasTool,
      clearAnnotationRef,
      hasVisiblePanes,
      keyboardTimeline: selection.keyboardTimeline,
      onSelectedTrackChange,
      setCanvasTool,
    });

  // Keep the transport zoom control stable between zoom changes.
  const zoomControl = useMemo(
    () => <PreviewZoomField onChange={requestZoom} zoomPercent={zoomPercent} />,
    [requestZoom, zoomPercent],
  );

  const { toggleCursorPanel, toggleKeyboardPanel } = useRecordingToolbar({
    canEditActiveTrack,
    canResizeActiveTrack,
    canvasTool,
    changeCanvasTool,
    hasCursorData,
    hasKeyboardData,
    hasVisiblePanes,
    isPlaying,
  });
  useEffect(() => {
    playhead.publish(0, 0);
  }, [artifactId, playhead]);

  useRecordingPreviewShortcuts({
    annotations,
    canMoveActiveVideoTrack,
    canResizeActiveTrack,
    canvasTool,
    hasCursorData,
    hasKeyboardData,
    hasVisiblePanes,
    isCropping,
    layout,
    leaveCropTool,
    moveActiveVideoTrackBackward,
    moveActiveVideoTrackForward,
    nudgeActiveTrack: selection.nudgeActiveTrack,
    player,
    step: transport.timelineBlade.step,
    toggleCursorPanel,
    toggleKeyboardPanel,
    toggleTool,
  });
  const changeEnabledTracks = useCallback(
    (tracks: Set<number>) => {
      onEnabledTracksChange?.([...tracks]);
    },
    [onEnabledTracksChange],
  );
  const changeEnabledVideoTracks = useCallback(
    (tracks: Set<RecordingVideoTrackId>) => {
      onEnabledVideoTracksChange?.([...tracks]);
    },
    [onEnabledVideoTracksChange],
  );
  const changeSelectedTrack = useRecordingTrackSelection({
    clearAnnotations: clearAnnotationRef,
    clearKeyboard: selection.keyboardTimeline.selection.onClear,
    onSelectedTrackChange,
    setTool: changeCanvasTool,
  });
  const { copyCurrentFrame, copyError } = useCopyRecordingFrame({
    annotationClips: annotations.clips,
    artifactId,
    bakeCamera,
    cameraOverlay,
    cursorEffects,
    getPositionMs: player.getPositionMs,
    keyboardEffects,
    recordingOutput: effectiveRecordingOutput,
  });

  return (
    <div className="flex min-h-0 grow flex-col">
      <RecordingPreviewStage
        {...props}
        {...transport}
        activeRecordingOutput={activeRecordingOutput}
        audioVolumeByStream={audioVolumeByStream}
        canPreviewBakedCamera={canPreviewBakedCamera}
        canvasTool={canvasTool}
        copyError={copyError}
        enabledTracks={enabledTracks}
        screenCanvasRef={screenCanvasRef}
      />

      {layout ? (
        <RecordingPreviewTimelineBand
          {...props}
          {...transport}
          audioVolumeByStream={audioVolumeByStream}
          changeCanvasTool={changeCanvasTool}
          changeEnabledTracks={changeEnabledTracks}
          changeEnabledVideoTracks={changeEnabledVideoTracks}
          changeSelectedTrack={changeSelectedTrack}
          copyCurrentFrame={copyCurrentFrame}
          enabledTracks={enabledTracks}
          keyboardTimeline={selection.keyboardTimeline}
          layout={layout}
          playhead={playhead}
          selectedVideoTracks={selectedVideoTracks}
          videoTrackOrderList={videoTrackOrderList}
          zoomControl={zoomControl}
        />
      ) : null}
    </div>
  );
}
