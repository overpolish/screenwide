// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { CircularProgress } from "../../../components/base/circular-progress/circular-progress";
import { Text } from "../../../components/base/text/text";
import { uncroppedCameraPreviewOverlay } from "../camera-overlay-geometry";
import { usePublishKeyboardShortcut } from "../keyboard-shortcut-channel";
import { useRecenterInsetControls } from "../recenter-inset-channel";
import {
  DEFAULT_KEYBOARD_EFFECTS,
  defaultCameraOverlay,
} from "../recording-export-settings";
import { createRecordingTimelineEdit } from "../recording-timeline-edit";
import { uncroppedScreenshotPreviewOutput } from "../screenshot-crop";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
  defaultScreenshotOutput,
  defaultRecordingOutput,
  recordingVideoTrackOrder,
} from "../screenshot-output";
import { EditorToolId } from "../tool-panels/tool-registry";
import { useCanvasTool } from "../tool-panels/use-canvas-tool";
import { useToolPanelFollowsTool } from "../tool-panels/use-tool-panel-follows-tool";
import { RecordingVideoTrackId } from "../types";
import { useCopyRecordingFrame } from "../use-copy-recording-frame";
import { useEditorEditGesture } from "../use-editor-edit-history";
import { useEditorWindowShortcuts } from "../use-editor-window-shortcuts";
import { usePreviewZoom } from "../use-preview-zoom";
import { useRecordingAnnotations } from "../use-recording-annotations";
import { useRecordingPreviewPlayer } from "../use-recording-preview-player";
import { useRecordingTimelineThumbnails } from "../use-recording-timeline-thumbnails";

import { BakedCameraPreviewViewport } from "./baked-camera-preview-viewport";
import { NativeAudioRibbon } from "./native-audio-ribbon";
import { useRegisterPreviewFit } from "./preview-fit-context";
import { PreviewZoomField } from "./preview-readouts";
import { RecordingCanvasTool } from "./recording-crop-toggle";
import { RecordingOutputPreviewViewport } from "./recording-output-preview-viewport";
import { RecordingPlaybackControls } from "./recording-playback-controls";
import { RECORDING_PREVIEW_PANE_GAP } from "./recording-preview-layout";
import { RecordingPreviewViewport } from "./recording-preview-viewport";
import {
  recordingVideoSelectionOverlay,
  recordingVideoSelectionTargets,
} from "./recording-selection-overlay";
import { RecordingTrackLanes } from "./recording-track-lanes";
import { ResizableRecordingTimelineArea } from "./resizable-recording-timeline-area";
import { createPlayhead } from "./scrub-playhead";
import { useRecordingCanvasContextMenu } from "./use-recording-canvas-context-menu";
import {
  KEYBOARD_LAYER_ID,
  useRecordingKeyboardPreviewEditing,
} from "./use-recording-keyboard-canvas-editing";
import { useRecordingRecenter } from "./use-recording-recenter";
import { useRecordingSelectionGesture } from "./use-recording-selection-gesture";
import { useRecordingSelectionNudge } from "./use-recording-selection-nudge";
import { useRecordingTimelineBlade } from "./use-recording-timeline-blade";
import { useRecordingToolbar } from "./use-recording-toolbar";
import { useRecordingTrackSelection } from "./use-recording-track-selection";
import { useRecordingTrimPreview } from "./use-recording-trim-preview";

import type { ScrubPreviewProps } from "./scrub-preview";

const EMPTY_AUDIO_TRACKS: NonNullable<ScrubPreviewProps["audioTracks"]> = [];

/** A normalized coordinate as the panel's percent field shows it, held to two
 * decimals so a redrawn frame is not published as a new value. */
const roundedPercent = (value: number) => Math.round(value * 10000) / 100;

/** The toolbar's own name for a tool, in the registry's vocabulary. */
const recordingToolId = (tool: RecordingCanvasTool): EditorToolId | null =>
  tool === "canvas" ? "frame" : tool;

/** One line of trouble under the preview: the audio, the player and the frame
 * copier each report their own, and each reads the same way. */
function PreviewError({ message }: { message?: string | null }) {
  if (!message) return null;
  return (
    <Text
      className="px-window-inset pb-control-inset text-error"
      variant="footnote"
    >
      {message}
    </Text>
  );
}

/** Editor playback whose decode, audio output and timeline are all owned by Rust. */
export function NativeRecordingPreview({
  artifactId,
  audioError,
  audioTrackVolumes = [],
  audioTracks = EMPTY_AUDIO_TRACKS,
  bakeCamera = false,
  cameraOverlay = defaultCameraOverlay(),
  cursorEffects = {
    bake: true,
    clickAnimation: true,
    clipAtVideoEdge: false,
    motionBlur: true,
    sizePercent: 100,
    smoothMovement: true,
  },
  durationMs,
  enabledStreamIndices,
  enabledVideoTracks = [],
  hasCursorData = false,
  hasKeyboardData = false,
  isExportOpen = false,
  isPreparingAudio = false,
  isPreparingPreview = false,
  isSaving = false,
  keyboardEffects = DEFAULT_KEYBOARD_EFFECTS,
  keyboardMaximumWidthUnits,
  onCameraOverlayChange,
  onEnabledTracksChange,
  onEnabledVideoTracksChange,
  onKeyboardEffectsChange,
  onRecordingOutputChange,
  onRecordingTimelineEditChange,
  onSelectedTrackChange,
  onVideoTrackOrderChange,
  previewLayout,
  previewOutputDimensions,
  previewSourceDimensions,
  recordingOutput,
  recordingTimelineEdit,
  selectedTrack = null,
}: ScrubPreviewProps) {
  const screenCanvasRef = useRef<HTMLCanvasElement>(null);
  const cameraCanvasRef = useRef<HTMLCanvasElement>(null);
  const recenterRefreshRef = useRef<
    (crop: ScreenshotOutputSettings["sourceCrop"]) => void
  >(() => undefined);
  const editGesture = useEditorEditGesture();
  const totalDurationRef = useRef(durationMs);
  const [playhead] = useState(createPlayhead);
  const trimPreview = useRecordingTrimPreview({
    edit: recordingTimelineEdit,
    playhead,
    totalDurationRef,
  });
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
  // Everything derived below feeds memoized children. A canvas-resize gesture
  // re-renders this component at pointer rate, so a derived array or Set rebuilt
  // per render would defeat the memo of every subtree it reaches.
  const selectedStreamIndices = useMemo(
    () => enabledStreamIndices ?? audioTracks.map((track) => track.streamIndex),
    [audioTracks, enabledStreamIndices],
  );
  const enabledTracks = useMemo(
    () => new Set(selectedStreamIndices),
    [selectedStreamIndices],
  );
  const selectedVideoTracks = useMemo(
    () => new Set(enabledVideoTracks),
    [enabledVideoTracks],
  );
  const canPreviewBakedCamera =
    bakeCamera &&
    selectedVideoTracks.has("primary") &&
    selectedVideoTracks.has("camera");
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
  // are exactly where the user left them when the overlay goes away. The
  // export options window covers the workspace with the same kind of DOM blur,
  // and is suspended for the same reason.
  const isEditorSuspended =
    nativeEditorOwnsLayout && (isSaving || isExportOpen);
  const activeVideoTrack =
    selectedTrack === "primary" || selectedTrack === "camera"
      ? selectedTrack
      : null;
  const audioVolumeByStream = useMemo(
    () =>
      new Map(
        audioTrackVolumes.map(({ decibels, streamIndex }) => [
          streamIndex,
          decibels,
        ]),
      ),
    [audioTrackVolumes],
  );
  const activeRecordingOutput =
    (canvasTool === "canvas" ? canvasResizeDraft : null) ?? recordingOutput;
  const effectiveRecordingOutput =
    activeRecordingOutput ??
    defaultRecordingOutput({
      camera: previewOutputDimensions?.camera,
      primary: previewOutputDimensions?.primary ?? { height: 64, width: 64 },
    });
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
  // Resizing never changes layer order, so keep its identity across the drag.
  const videoTrackOrder = useMemo(
    () => recordingVideoTrackOrder(effectiveRecordingOutput),
    // eslint-disable-next-line @eslint-react/exhaustive-deps
    [effectiveRecordingOutput.cameraOnTop],
  );
  const videoTrackOrderList = useMemo(
    () => [...videoTrackOrder],
    [videoTrackOrder],
  );
  const cropSource =
    activeVideoTrack === "primary"
      ? previewSourceDimensions.primary
      : previewSourceDimensions.camera;
  const previewRecordingOutput = useMemo(() => {
    if (canvasTool !== "crop" || !activeVideoTrack || !cropSource)
      return effectiveRecordingOutput;
    if (bakeCamera && activeVideoTrack === "camera")
      return effectiveRecordingOutput;
    return {
      ...effectiveRecordingOutput,
      [activeVideoTrack]: uncroppedScreenshotPreviewOutput(
        cropSource,
        effectiveRecordingOutput[activeVideoTrack],
      ),
    };
  }, [
    activeVideoTrack,
    bakeCamera,
    canvasTool,
    cropSource,
    effectiveRecordingOutput,
  ]);
  const previewCameraOverlay = useMemo(() => {
    if (
      canvasTool !== "crop" ||
      activeVideoTrack !== "camera" ||
      !bakeCamera ||
      !previewSourceDimensions.camera
    )
      return cameraOverlay;
    const output = effectiveRecordingOutput.primary;
    return uncroppedCameraPreviewOverlay(
      {
        height: output.height,
        kind: "screen",
        sourceHeight: output.height,
        sourceWidth: output.width,
        width: output.width,
        x: 0,
        y: 0,
      },
      {
        height: previewSourceDimensions.camera.height,
        kind: "camera",
        sourceHeight: previewSourceDimensions.camera.height,
        sourceWidth: previewSourceDimensions.camera.width,
        width: previewSourceDimensions.camera.width,
        x: 0,
        y: 0,
      },
      cameraOverlay,
    );
  }, [
    activeVideoTrack,
    bakeCamera,
    cameraOverlay,
    canvasTool,
    effectiveRecordingOutput.primary,
    previewSourceDimensions.camera,
  ]);
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
  const clearAnnotationRef = useRef(() => {});
  const player = useRecordingPreviewPlayer({
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
    onSelectionGesture: selectionGesture.applyGesture,
    onZoomChange: reportZoom,
    recordingOutput: previewRecordingOutput,
    screenCanvasRef,
    selection: selectionOverlay,
    selectionTargets,
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
    isPlaying: player.isPlaying,
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
    tool: bakeCamera && activeVideoTrack === "camera" ? null : canvasTool,
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
  }, [getPlayerPositionMs, isPlaying]);
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
      pause();
      recenter.begin();
    },
    prepare: () => {
      pause();
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
    playhead,
    seekPlayer: player.seek,
    shortcutsEnabled: Boolean(layout) && !annotations.canDelete,
    totalDurationMs,
  });
  const canvasRefs = useMemo(
    () => [screenCanvasRef, cameraCanvasRef],
    [cameraCanvasRef, screenCanvasRef],
  );
  const visiblePaneEntries = useMemo(
    () =>
      layout?.panes
        .map((pane, index) => ({
          canvasRef: canvasRefs[index],
          pane,
          trackId: index === 0 ? ("primary" as const) : ("camera" as const),
        }))
        .filter(({ trackId }) => selectedVideoTracks.has(trackId))
        .sort(
          (left, right) =>
            videoTrackOrder.indexOf(left.trackId) -
            videoTrackOrder.indexOf(right.trackId),
        ) ?? [],
    [canvasRefs, layout, selectedVideoTracks, videoTrackOrder],
  );
  const visibleLayout = useMemo(() => {
    if (!layout) return null;
    const height = visiblePaneEntries.reduce(
      (maximum, { pane }) => Math.max(maximum, pane.height),
      0,
    );
    let x = 0;
    const panes = visiblePaneEntries.map(({ pane }) => {
      const visiblePane = { ...pane, x, y: (height - pane.height) / 2 };
      x += pane.width + RECORDING_PREVIEW_PANE_GAP;
      return visiblePane;
    });
    return {
      height,
      panes,
      width: Math.max(0, x - RECORDING_PREVIEW_PANE_GAP),
    };
  }, [layout, visiblePaneEntries]);
  const visibleCanvasRefs = useMemo(
    () => visiblePaneEntries.map(({ canvasRef }) => canvasRef),
    [visiblePaneEntries],
  );
  const screenPane = layout?.panes[0];
  const cameraPane = layout?.panes[1];
  const pause = player.pause;
  const play = player.play;
  const togglePlayback = useCallback(() => {
    if (isPlaying) pause();
    else play();
  }, [isPlaying, pause, play]);
  const canEditActiveTrack =
    activeVideoTrack !== null && selectedVideoTracks.has(activeVideoTrack);
  const canResizeActiveTrack =
    canEditActiveTrack && (!bakeCamera || canPreviewBakedCamera);
  const moveVideoTrack = useCallback(
    (track: RecordingVideoTrackId, direction: "backward" | "forward") => {
      const currentIndex = videoTrackOrder.indexOf(track);
      const nextIndex =
        direction === "forward" ? currentIndex - 1 : currentIndex + 1;
      if (
        currentIndex < 0 ||
        nextIndex < 0 ||
        nextIndex >= videoTrackOrder.length
      )
        return;
      const next = [...videoTrackOrder];
      [next[currentIndex], next[nextIndex]] = [
        next[nextIndex],
        next[currentIndex],
      ];
      onVideoTrackOrderChange?.(next);
    },
    [onVideoTrackOrderChange, videoTrackOrder],
  );
  const moveActiveVideoTrack = useCallback(
    (direction: "backward" | "forward") => {
      if (activeVideoTrack) moveVideoTrack(activeVideoTrack, direction);
    },
    [activeVideoTrack, moveVideoTrack],
  );
  // The shortcut hook re-binds its window listener whenever a handler identity
  // changes, so these stay stable across the per-move draft renders.
  const canMoveActiveVideoTrack =
    activeVideoTrack !== null && selectedVideoTracks.has(activeVideoTrack);
  const hasVisiblePanes = visiblePaneEntries.length > 0;
  const moveActiveVideoTrackBackward = useCallback(() => {
    moveActiveVideoTrack("backward");
  }, [moveActiveVideoTrack]);
  const moveActiveVideoTrackForward = useCallback(() => {
    moveActiveVideoTrack("forward");
  }, [moveActiveVideoTrack]);
  const canvasToolRef = useRef(canvasTool);
  canvasToolRef.current = canvasTool;
  useRecordingCanvasContextMenu({
    canvasTool,
    moveVideoTrack,
    onSelectedTrackChange,
    videoTrackOrderList,
    visiblePaneEntries,
  });
  const changeCanvasTool = useCallback(
    (next: RecordingCanvasTool) => {
      if (next === "arrow") {
        keyboardTimeline.selection.onClear();
        onSelectedTrackChange?.(
          bakeCamera ? "primary" : (activeVideoTrack ?? "primary"),
        );
      } else if (next !== "select") clearAnnotationRef.current();
      setCanvasTool(next);
    },
    [
      setCanvasTool,
      keyboardTimeline.selection,
      onSelectedTrackChange,
      bakeCamera,
      activeVideoTrack,
    ],
  );
  // A canvas tool acts on the panes on screen. When the last video track is
  // switched off there is nothing left for it to act on, so the tool is put
  // down and its panel goes with it rather than staying up over the ribbon
  // saying nothing is selected.
  useEffect(() => {
    if (!hasVisiblePanes && canvasToolRef.current !== null) setCanvasTool(null);
  }, [hasVisiblePanes, setCanvasTool]);
  // One toggle apiece, built from one function: picking up the tool already in
  // hand puts it down. A new canvas tool joins the record rather than copying
  // the body a fifth time.
  const toggleTool = useMemo(() => {
    const toggle = (tool: RecordingCanvasTool) => () => {
      changeCanvasTool(canvasToolRef.current === tool ? null : tool);
    };
    return {
      arrow: toggle("arrow"),
      canvas: toggle("canvas"),
      crop: toggle("crop"),
      select: toggle("select"),
    };
  }, [changeCanvasTool]);
  // The crop tool is the one tool you are "in": Enter accepts what is framed
  // and Escape backs out of it, and both simply put the tool down - the crop
  // itself was committed as each handle was released. While it is in hand that
  // Escape outranks the timeline's own deselects.
  const isCropping = canvasTool === "crop";
  const leaveCropTool = useCallback(() => {
    if (canvasToolRef.current === "crop") changeCanvasTool(null);
  }, [changeCanvasTool]);

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

  const canNudgeActiveTrack =
    canvasTool === "select" &&
    canMoveActiveVideoTrack &&
    !isPlaying &&
    !annotations.hasSelection;
  useEditorWindowShortcuts({
    onArrowTool: hasVisiblePanes && !isPlaying ? toggleTool.arrow : undefined,
    onConfirm: isCropping ? leaveCropTool : undefined,
    onDelete: annotations.canDelete ? annotations.deleteTargeted : undefined,
    onDeselect: annotations.hasSelection
      ? annotations.clearSelection
      : isCropping
        ? leaveCropTool
        : undefined,
    onMoveBackward: canMoveActiveVideoTrack
      ? moveActiveVideoTrackBackward
      : undefined,
    onMoveForward: canMoveActiveVideoTrack
      ? moveActiveVideoTrackForward
      : undefined,
    onNudge: canNudgeActiveTrack ? nudgeActiveTrack : undefined,
    onResizeCanvas: canResizeActiveTrack ? toggleTool.canvas : undefined,
    onSelectTool: hasVisiblePanes ? toggleTool.select : undefined,
    onStep: !canNudgeActiveTrack && layout ? timelineBlade.step : undefined,
    onToggleCrop: hasVisiblePanes ? toggleTool.crop : undefined,
    onToggleCursorPanel: hasCursorData ? toggleCursorPanel : undefined,
    onToggleKeyboardPanel: hasKeyboardData ? toggleKeyboardPanel : undefined,
    onTogglePlayback: layout ? togglePlayback : undefined,
    ownsEscape: isCropping || annotations.hasSelection,
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
    clearKeyboard: keyboardTimeline.selection.onClear,
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
      <section className="relative flex min-h-0 min-w-0 grow flex-col">
        <div className="flex min-h-0 grow items-stretch justify-center">
          {!layout ? (
            <div className="flex grow items-center justify-center gap-3 text-xs text-muted">
              <CircularProgress
                aria-label="Preparing recording preview"
                isIndeterminate
              />
              Preparing recording preview
            </div>
          ) : canPreviewBakedCamera && screenPane && cameraPane ? (
            <div className="flex min-h-0 min-w-0 grow flex-col">
              <BakedCameraPreviewViewport
                isBusy={
                  previewLayout === undefined &&
                  (player.isPreparing || isPreparingPreview)
                }
                outputSettings={
                  // Draft output keeps the frame and workspace on the resize
                  // before it reaches the editor window's state.
                  activeRecordingOutput?.primary ??
                  defaultScreenshotOutput(screenPane.width, screenPane.height)
                }
                screenCanvasRef={screenCanvasRef}
                tool={canvasTool}
              />
            </div>
          ) : visibleLayout &&
            visibleLayout.panes.length > 0 &&
            activeRecordingOutput ? (
            <div className="flex min-h-0 min-w-0 grow flex-col">
              <RecordingOutputPreviewViewport
                entries={visiblePaneEntries}
                outputs={activeRecordingOutput}
                tool={canvasTool}
              />
            </div>
          ) : visibleLayout && visibleLayout.panes.length > 0 ? (
            <div className="flex min-h-0 min-w-0 grow flex-col">
              <RecordingPreviewViewport
                canvasRefs={visibleCanvasRefs}
                isBusy={
                  previewLayout === undefined &&
                  (player.isPreparing || isPreparingPreview)
                }
                layout={visibleLayout}
              />
            </div>
          ) : (
            <NativeAudioRibbon
              artifactId={artifactId}
              audioTracks={audioTracks}
              enabledTracks={enabledTracks}
              volumes={audioVolumeByStream}
            />
          )}
        </div>

        <PreviewError message={audioError} />
        <PreviewError message={player.error} />
        <PreviewError message={copyError} />
      </section>

      {layout ? (
        <ResizableRecordingTimelineArea
          artifactId={artifactId}
          header={
            <RecordingPlaybackControls
              durationMs={timelineBlade.timelineDurationMs}
              isPlaying={player.isPlaying}
              onCopyCurrentFrame={copyCurrentFrame}
              onPause={player.pause}
              onPlay={player.play}
              onPlaybackRateChange={player.setPlaybackRate}
              playbackRate={player.playbackRate}
              playhead={playhead}
              zoomControl={zoomControl}
            />
          }
          ready={!isPreparingAudio}
        >
          {isPreparingAudio ? (
            <div className="flex shrink-0 items-center justify-center gap-control-inset py-layout text-body text-content-fg-secondary">
              <CircularProgress
                aria-label="Preparing audio preview"
                isIndeterminate
                size="small"
              />
              Preparing audio tracks
            </div>
          ) : (
            <RecordingTrackLanes
              adjustedKeyboardFragmentIds={keyboardTimeline.adjustedFragmentIds}
              annotationClips={annotations.clips}
              audioTracks={audioTracks}
              blade={timelineBlade.blade}
              durationMs={timelineBlade.timelineDurationMs}
              enabledTracks={enabledTracks}
              enabledVideoTracks={selectedVideoTracks}
              hiddenKeyboardFragmentIds={keyboardTimeline.hiddenFragmentIds}
              hiddenKeyboardItemIds={keyboardTimeline.hiddenItemIds}
              // Shortcuts turned off leave nothing to place, so the lane goes
              // with them rather than showing items that are not drawn.
              keyboardItems={keyboardEffects.bake ? keyboardTimeline.items : []}
              keyboardSelection={keyboardTimeline.selection}
              layout={layout}
              onAnnotationsChange={annotations.onClipsChange}
              // Choosing a mark from its lane picks the Select tool up, the
              // way choosing a camera or screen clip does, so the mark is in
              // hand rather than merely highlighted.
              onAnnotationSelect={(id) => {
                annotations.onSelect(id);
                changeCanvasTool("select");
              }}
              onAnnotationsPreview={annotations.onPreviewClips}
              onEnabledTracksChange={changeEnabledTracks}
              onEnabledVideoTracksChange={changeEnabledVideoTracks}
              onSeek={timelineBlade.seek}
              onSelectedTrackChange={changeSelectedTrack}
              onVideoTrackOrderChange={onVideoTrackOrderChange}
              playhead={playhead}
              selectedAnnotationId={annotations.selectedId}
              selectedTrack={annotations.hasSelection ? null : selectedTrack}
              sourceDurationMs={durationMs}
              thumbnails={timelineThumbnails}
              videoTrackOrder={videoTrackOrderList}
              volumes={audioVolumeByStream}
            />
          )}
        </ResizableRecordingTimelineArea>
      ) : null}
    </div>
  );
}
