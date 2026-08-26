// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import { CircularProgressBar } from "../../../components/base/circular-progress-bar/circular-progress-bar";
import { copyRecordingPreviewFrameToClipboard } from "../api";
import {
  cameraOverlayGeometry,
  uncroppedCameraPreviewOverlay,
} from "../camera-overlay-geometry";
import {
  DEFAULT_KEYBOARD_EFFECTS,
  defaultCameraOverlay,
} from "../recording-export-settings";
import {
  applyScreenshotCropGesture,
  commitScreenshotCrop,
  uncroppedScreenshotPreviewOutput,
} from "../screenshot-crop";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
  defaultScreenshotOutput,
  defaultRecordingOutput,
  recordingVideoTrackOrder,
  resizeScreenshotWorkspaceCanvasEdges,
  screenshotOutputDimensions,
  screenshotWorkspaceItemOutput,
} from "../screenshot-output";
import { applyScreenshotRecenterGesture } from "../screenshot-recenter";
import {
  CameraOverlaySettings,
  RecordingTrackId,
  RecordingVideoTrackId,
} from "../types";
import { useExportEditGesture } from "../use-export-edit-history";
import { useExportWindowShortcuts } from "../use-export-window-shortcuts";
import { useRecordingPreviewPlayer } from "../use-recording-preview-player";
import { useRecordingTimelineThumbnails } from "../use-recording-timeline-thumbnails";

import { AudioVisualizer } from "./audio-visualizer";
import { BakedCameraPreviewViewport } from "./baked-camera-preview-viewport";
import { PreviewToolbar } from "./preview-toolbar";
import {
  RecordingCanvasTools,
  RecordingCanvasTool,
} from "./recording-crop-toggle";
import { usePublishRecordingOutputDimensions } from "./recording-output-dimensions-channel";
import { RecordingOutputPreviewViewport } from "./recording-output-preview-viewport";
import { RecordingPlaybackControls } from "./recording-playback-controls";
import { RECORDING_PREVIEW_PANE_GAP } from "./recording-preview-layout";
import { RecordingPreviewViewport } from "./recording-preview-viewport";
import { normalizedRecordingSelection } from "./recording-selection";
import { RecordingTrackLanes } from "./recording-track-lanes";
import { createPlayhead } from "./scrub-playhead";
import {
  KEYBOARD_LAYER_ID,
  useRecordingKeyboardPreviewEditing,
} from "./use-recording-keyboard-canvas-editing";
import { useRecordingRecenter } from "./use-recording-recenter";
import { useRecordingSelectionNudge } from "./use-recording-selection-nudge";
import { useRecordingTimelineBlade } from "./use-recording-timeline-blade";
import { useRecordingTrimPreview } from "./use-recording-trim-preview";

import type { RecordingSelectionGestureEvent } from "../use-recording-preview-surface";
import type { ScrubPreviewProps } from "./scrub-preview";

const EMPTY_AUDIO_TRACKS: NonNullable<ScrubPreviewProps["audioTracks"]> = [];
const FRAME_LAYER_ID = 0xffffffff;
const AUTO_FIT_MOVE_EDGE = 1 << 17;
const AUTO_FIT_COMMIT_EDGE = 1 << 18;

/** Export playback whose decode, audio output and timeline are all owned by Rust. */
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
  hasKeyboardData = false,
  inspector,
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
}: ScrubPreviewProps & { inspector?: ReactNode }) {
  const screenCanvasRef = useRef<HTMLCanvasElement>(null);
  const cameraCanvasRef = useRef<HTMLCanvasElement>(null);
  const selectionGestureRef = useRef<{
    cameraOverlaySnapshot: CameraOverlaySettings | null;
    lastDeltaX: number;
    lastDeltaY: number;
    lastScale: number;
    operation: RecordingSelectionGestureEvent["operation"];
    outputSnapshot: RecordingOutputSettings[RecordingVideoTrackId] | null;
    paneIndex: number;
    recenterMode: boolean;
    trackId: RecordingVideoTrackId;
  } | null>(null);
  const recenterActionRef = useRef<() => void>(() => undefined);
  const recenterRefreshRef = useRef<
    (crop: ScreenshotOutputSettings["sourceCrop"]) => void
  >(() => undefined);
  const editGesture = useExportEditGesture();
  const totalDurationRef = useRef(durationMs);
  const [playhead] = useState(createPlayhead);
  const trimPreview = useRecordingTrimPreview({
    edit: recordingTimelineEdit,
    playhead,
    totalDurationRef,
  });
  const [zoomPercent, setZoomPercent] = useState(100);
  const [canvasTool, setCanvasTool] = useState<RecordingCanvasTool>("select");
  // A canvas resize runs at pointer rate; committing every move to the export
  // window's state re-renders the inspector, lanes and timeline and starves
  // the native pane's layout loop. The gesture renders from this draft and
  // commits once on release, exactly like the screenshot editor.
  const [canvasResizeDraft, setCanvasResizeDraft] =
    useState<RecordingOutputSettings | null>(null);
  const [copyError, setCopyError] = useState<string | null>(null);
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
  // instead. Suspending the native editor for the duration of the save (every
  // path that clears `isSaving`: success, failure and cancel) hands input back
  // to the webview without changing which side owns the layout, so the panes
  // keep rendering natively and the selection chrome hides itself while the
  // editor is inactive.
  const isEditorSuspended = nativeEditorOwnsLayout && isSaving;
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
  usePublishRecordingOutputDimensions(effectiveRecordingOutput.primary);
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
  const videoSelectionOverlay = useMemo(() => {
    if (canvasTool === "canvas") {
      // Frame is a synthetic selection, just as in the screenshot workspace.
      // In baked mode it always belongs to the primary output; in split mode
      // it belongs to the currently selected video frame.
      const frameTrack = canPreviewBakedCamera ? "primary" : activeVideoTrack;
      if (!frameTrack || !selectedVideoTracks.has(frameTrack)) return null;
      return {
        layerId: FRAME_LAYER_ID,
        paneIndex: frameTrack === "primary" ? 0 : 1,
        radiusPercent: 0,
        rect: { height: 1, width: 1, x: 0, y: 0 },
      };
    }
    if (
      (canvasTool !== "select" &&
        canvasTool !== "crop" &&
        canvasTool !== "recenter") ||
      !activeVideoTrack ||
      (canvasTool === "recenter" && activeVideoTrack !== "primary") ||
      !selectedVideoTracks.has(activeVideoTrack)
    )
      return null;
    const primaryOutput = screenshotOutputDimensions(
      effectiveRecordingOutput.primary,
    );
    const primarySource = previewSourceDimensions.primary;
    if (!primarySource) return null;
    if (canPreviewBakedCamera) {
      if (activeVideoTrack === "primary") {
        return normalizedRecordingSelection({
          mode: canvasTool,
          output: effectiveRecordingOutput.primary,
          paneIndex: 0,
          source: primarySource,
        });
      }
      const cameraSource = previewSourceDimensions.camera;
      if (!cameraSource) return null;
      const geometry = cameraOverlayGeometry(
        {
          height: primaryOutput.height,
          kind: "screen",
          sourceHeight: primarySource.height,
          sourceWidth: primarySource.width,
          width: primaryOutput.width,
          x: 0,
          y: 0,
        },
        {
          height: cameraSource.height,
          kind: "camera",
          sourceHeight: cameraSource.height,
          sourceWidth: cameraSource.width,
          width: cameraSource.width,
          x: 0,
          y: 0,
        },
        cameraOverlay,
      );
      return {
        cropMode: canvasTool === "crop",
        image: {
          height: geometry.camera.height / Math.max(1, primaryOutput.height),
          width: geometry.camera.width / Math.max(1, primaryOutput.width),
          x: geometry.camera.x / Math.max(1, primaryOutput.width),
          y: geometry.camera.y / Math.max(1, primaryOutput.height),
        },
        layerId: 1,
        paneIndex: 0,
        radiusPercent: cameraOverlay.radiusPercent,
        rect: {
          height: geometry.frame.height / Math.max(1, primaryOutput.height),
          width: geometry.frame.width / Math.max(1, primaryOutput.width),
          x: geometry.frame.x / Math.max(1, primaryOutput.width),
          y: geometry.frame.y / Math.max(1, primaryOutput.height),
        },
      };
    }
    const paneIndex = activeVideoTrack === "primary" ? 0 : 1;
    const source =
      activeVideoTrack === "primary"
        ? primarySource
        : previewSourceDimensions.camera;
    if (!source) return null;
    return normalizedRecordingSelection({
      mode: canvasTool,
      output: effectiveRecordingOutput[activeVideoTrack],
      paneIndex,
      source,
    });
  }, [
    activeVideoTrack,
    canPreviewBakedCamera,
    cameraOverlay,
    canvasTool,
    effectiveRecordingOutput,
    previewSourceDimensions.camera,
    previewSourceDimensions.primary,
    selectedVideoTracks,
  ]);
  const videoSelectionTargets = useMemo(() => {
    if (canvasTool === "canvas")
      return (["primary", "camera"] as const).flatMap((trackId) =>
        selectedVideoTracks.has(trackId) && previewSourceDimensions[trackId]
          ? [
              {
                layerId: FRAME_LAYER_ID,
                paneIndex: trackId === "primary" ? 0 : 1,
                radiusPercent: 0,
                rect: { height: 1, width: 1, x: 0, y: 0 },
              },
            ]
          : [],
      );
    if (canvasTool === "recenter") {
      const source = previewSourceDimensions.primary;
      if (!source || !selectedVideoTracks.has("primary")) return null;
      return [
        normalizedRecordingSelection({
          mode: "recenter",
          output: effectiveRecordingOutput.primary,
          paneIndex: 0,
          source,
        }),
      ];
    }
    if (canvasTool !== "select" && canvasTool !== "crop") return null;
    if (canPreviewBakedCamera) {
      const primarySource = previewSourceDimensions.primary;
      const cameraSource = previewSourceDimensions.camera;
      if (!primarySource || !cameraSource) return null;
      const output = screenshotOutputDimensions(
        effectiveRecordingOutput.primary,
      );
      const cameraGeometry = cameraOverlayGeometry(
        {
          height: output.height,
          kind: "screen",
          sourceHeight: primarySource.height,
          sourceWidth: primarySource.width,
          width: output.width,
          x: 0,
          y: 0,
        },
        {
          height: cameraSource.height,
          kind: "camera",
          sourceHeight: cameraSource.height,
          sourceWidth: cameraSource.width,
          width: cameraSource.width,
          x: 0,
          y: 0,
        },
        cameraOverlay,
      );
      return [
        normalizedRecordingSelection({
          mode: canvasTool,
          output: effectiveRecordingOutput.primary,
          paneIndex: 0,
          source: primarySource,
        }),
        {
          cropMode: canvasTool === "crop",
          image: {
            height: cameraGeometry.camera.height / Math.max(1, output.height),
            width: cameraGeometry.camera.width / Math.max(1, output.width),
            x: cameraGeometry.camera.x / Math.max(1, output.width),
            y: cameraGeometry.camera.y / Math.max(1, output.height),
          },
          layerId: 1,
          paneIndex: 0,
          radiusPercent: cameraOverlay.radiusPercent,
          rect: {
            height: cameraGeometry.frame.height / Math.max(1, output.height),
            width: cameraGeometry.frame.width / Math.max(1, output.width),
            x: cameraGeometry.frame.x / Math.max(1, output.width),
            y: cameraGeometry.frame.y / Math.max(1, output.height),
          },
        },
      ];
    }
    return (["primary", "camera"] as const).flatMap((trackId) => {
      if (!selectedVideoTracks.has(trackId)) return [];
      const source = previewSourceDimensions[trackId];
      if (!source) return [];
      return [
        normalizedRecordingSelection({
          mode: canvasTool,
          output: effectiveRecordingOutput[trackId],
          paneIndex: trackId === "primary" ? 0 : 1,
          source,
        }),
      ];
    });
  }, [
    canPreviewBakedCamera,
    cameraOverlay,
    canvasTool,
    effectiveRecordingOutput,
    previewSourceDimensions,
    selectedVideoTracks,
  ]);
  const keyboardSelection = keyboardPreview.selection;
  const selectionOverlay =
    keyboardSelection &&
    keyboardTimeline.selection.ids.has(
      visibleKeyboardFragment?.fragmentId ?? "",
    )
      ? keyboardSelection
      : videoSelectionOverlay;
  const selectionTargets = keyboardSelection
    ? [...(videoSelectionTargets ?? []), keyboardSelection]
    : videoSelectionTargets;
  const selectionGesture = (event: RecordingSelectionGestureEvent) => {
    if (keyboardCanvas.applyGesture(event)) return;
    if (event.operation === "recenterAction") {
      if (event.phase === "begin") recenterActionRef.current();
      return;
    }
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
    const isRecenterGesture =
      canvasTool === "recenter" &&
      trackId === "primary" &&
      (event.operation === "move" || event.operation === "resize");
    if (event.phase === "begin") {
      if (
        !trackId ||
        (isFrameGesture
          ? canvasTool !== "canvas"
          : isCropGesture
            ? canvasTool !== "crop"
            : !isRecenterGesture && canvasTool !== "select") ||
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
        recenterMode: isRecenterGesture,
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
      const start = active.cameraOverlaySnapshot;
      const frameX = start.frameXPercent + event.deltaX * 100;
      const frameY = start.frameYPercent + event.deltaY * 100;
      let next: CameraOverlaySettings;
      if (event.operation === "cropMove") {
        next = {
          ...start,
          frameXPercent: frameX,
          frameYPercent: frameY,
        };
      } else if (event.operation === "cropResize") {
        let left = start.frameXPercent;
        let top = start.frameYPercent;
        let right = left + start.frameWidthPercent;
        let bottom = top + start.frameHeightPercent;
        if ((event.edges & 1) !== 0) left += event.deltaX * 100;
        if ((event.edges & 2) !== 0) right += event.deltaX * 100;
        if ((event.edges & 4) !== 0) top += event.deltaY * 100;
        if ((event.edges & 8) !== 0) bottom += event.deltaY * 100;
        next = {
          ...start,
          frameHeightPercent: bottom - top,
          frameWidthPercent: right - left,
          frameXPercent: left,
          frameYPercent: top,
        };
      } else if (event.operation === "radius") {
        next = {
          ...start,
          radiusPercent: Math.min(50, Math.max(0, event.scale)),
        };
      } else if (event.operation === "resize") {
        const scale = Math.min(8, Math.max(0, event.scale));
        const transform = (
          value: number,
          startFrame: number,
          nextFrame: number,
        ) => {
          if (Math.abs(scale - 1) < 1e-9) return value;
          const anchor = (nextFrame - startFrame * scale) / (1 - scale);
          return anchor + (value - anchor) * scale;
        };
        next = {
          ...start,
          cameraWidthPercent: start.cameraWidthPercent * scale,
          cameraXPercent: transform(
            start.cameraXPercent,
            start.frameXPercent,
            frameX,
          ),
          cameraYPercent: transform(
            start.cameraYPercent,
            start.frameYPercent,
            frameY,
          ),
          frameHeightPercent: start.frameHeightPercent * scale,
          frameWidthPercent: start.frameWidthPercent * scale,
          frameXPercent: frameX,
          frameYPercent: frameY,
        };
      } else {
        next = {
          ...start,
          cameraXPercent: start.cameraXPercent + event.deltaX * 100,
          cameraYPercent: start.cameraYPercent + event.deltaY * 100,
          frameXPercent: frameX,
          frameYPercent: frameY,
        };
      }
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
    if (active.recenterMode) {
      const next = applyScreenshotRecenterGesture({
        deltaX: event.deltaX,
        deltaY: event.deltaY,
        edges: event.edges,
        operation: event.operation,
        scale: event.scale,
        settings: snapshot,
        source: previewSourceDimensions.primary,
      });
      if (next && shouldApply) onRecordingOutputChange?.("primary", next);
      if (event.phase === "update") finaliseGestureFrame();
      if (event.phase === "end") {
        selectionGestureRef.current = null;
        requestAnimationFrame(editGesture.endGesture);
      }
      return;
    }
    const cropX = snapshot.screenshotCropXPercent + event.deltaX * 100;
    const cropY = snapshot.screenshotCropYPercent + event.deltaY * 100;
    let next: RecordingOutputSettings[RecordingVideoTrackId];
    if (event.operation === "cropMove" || event.operation === "cropResize") {
      const source =
        active.trackId === "primary"
          ? previewSourceDimensions.primary
          : previewSourceDimensions.camera;
      if (!source) return;
      next = applyScreenshotCropGesture({
        ...event,
        operation: event.operation,
        output: screenshotOutputDimensions(snapshot),
        settings: snapshot,
        source,
      });
      if (event.phase === "end")
        next = commitScreenshotCrop(snapshot, next, source);
    } else if (event.operation === "frameResize") {
      const source =
        active.trackId === "primary"
          ? previewSourceDimensions.primary
          : previewSourceDimensions.camera;
      if (!source) return;
      const workspace = {
        ...snapshot,
        items: [{ id: 0, output: snapshot }],
      };
      const resized = resizeScreenshotWorkspaceCanvasEdges({
        deltaX: event.deltaX,
        deltaY: event.deltaY,
        edges: event.edges,
        settings: workspace,
        sources: [{ ...source, id: 0 }],
      });
      next = screenshotWorkspaceItemOutput(resized, 0);
    } else if (event.operation === "radius") {
      next = {
        ...snapshot,
        radiusPercent: Math.min(50, Math.max(0, event.scale)),
      };
    } else if (event.operation === "resize") {
      const scale = Math.min(8, Math.max(0, event.scale));
      const transform = (
        value: number,
        startFrame: number,
        nextFrame: number,
      ) => {
        if (Math.abs(scale - 1) < 1e-9) return value;
        const anchor = (nextFrame - startFrame * scale) / (1 - scale);
        return anchor + (value - anchor) * scale;
      };
      next = {
        ...snapshot,
        screenshotCropHeightPercent:
          snapshot.screenshotCropHeightPercent * scale,
        screenshotCropWidthPercent: snapshot.screenshotCropWidthPercent * scale,
        screenshotCropXPercent: cropX,
        screenshotCropYPercent: cropY,
        screenshotImageWidthPercent:
          snapshot.screenshotImageWidthPercent * scale,
        screenshotImageXPercent: transform(
          snapshot.screenshotImageXPercent,
          snapshot.screenshotCropXPercent,
          cropX,
        ),
        screenshotImageYPercent: transform(
          snapshot.screenshotImageYPercent,
          snapshot.screenshotCropYPercent,
          cropY,
        ),
      };
    } else {
      next = {
        ...snapshot,
        screenshotCropXPercent: cropX,
        screenshotCropYPercent: cropY,
        screenshotImageXPercent:
          snapshot.screenshotImageXPercent + event.deltaX * 100,
        screenshotImageYPercent:
          snapshot.screenshotImageYPercent + event.deltaY * 100,
      };
    }
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
  const editsBakedCameraOverlay =
    canPreviewBakedCamera && activeVideoTrack === "camera";
  const nudgeActiveTrack = useRecordingSelectionNudge({
    activeTrack: activeVideoTrack,
    applyGesture: selectionGesture,
    cameraOverlay,
    editsBakedCamera: editsBakedCameraOverlay,
    gestureAccepted: () => selectionGestureRef.current !== null,
    output: effectiveRecordingOutput,
    outputDimensions: previewOutputDimensions,
  });
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
      if (paneIndex === KEYBOARD_LAYER_ID) {
        keyboardCanvas.selectVisible();
        onSelectedTrackChange?.(null);
        return;
      }
      keyboardTimeline.selection.onClear();
      const trackId = paneIndex === 0 ? "primary" : "camera";
      if (selectedVideoTracks.has(trackId)) onSelectedTrackChange?.(trackId);
    },
    onSelectionGesture: selectionGesture,
    onZoomChange: setZoomPercent,
    recordingOutput: previewRecordingOutput,
    screenCanvasRef,
    selection: selectionOverlay,
    selectionTargets,
    sourceDurationMs: durationMs,
    timelineEdit: recordingTimelineEdit,
    zoomPercent,
  });
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
  recenterActionRef.current = recenter.begin;
  recenterRefreshRef.current = recenter.refresh;
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
    shortcutsEnabled: Boolean(layout),
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
  const canRecenterPrimary =
    activeVideoTrack === "primary" &&
    selectedVideoTracks.has("primary") &&
    screenPane?.kind === "screen";
  const moveActiveVideoTrack = useCallback(
    (direction: "backward" | "forward") => {
      if (!activeVideoTrack) return;
      const currentIndex = videoTrackOrder.indexOf(activeVideoTrack);
      const nextIndex =
        direction === "forward" ? currentIndex - 1 : currentIndex + 1;
      if (nextIndex < 0 || nextIndex >= videoTrackOrder.length) return;
      const next = [...videoTrackOrder];
      [next[currentIndex], next[nextIndex]] = [
        next[nextIndex],
        next[currentIndex],
      ];
      onVideoTrackOrderChange?.(next);
    },
    [activeVideoTrack, onVideoTrackOrderChange, videoTrackOrder],
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
  const toggleCanvasTool = useCallback(() => {
    setCanvasTool((current) => (current === "canvas" ? null : "canvas"));
  }, []);
  const toggleSelectTool = useCallback(() => {
    setCanvasTool((current) => (current === "select" ? null : "select"));
  }, []);
  const toggleCropTool = useCallback(() => {
    setCanvasTool((current) => (current === "crop" ? null : "crop"));
  }, []);
  const toggleRecenterTool = useCallback(() => {
    pause();
    if (canvasTool !== "recenter") recenter.prepare();
    setCanvasTool((current) => (current === "recenter" ? null : "recenter"));
  }, [canvasTool, pause, recenter]);
  const changeCanvasTool = useCallback(
    (next: RecordingCanvasTool) => {
      if (next === "recenter") {
        pause();
        recenter.prepare();
      }
      setCanvasTool(next);
    },
    [pause, recenter],
  );

  // The tools read the committed `recordingOutput`, never the resize draft, so
  // holding the element keeps the memoized toolbar's props stable mid-gesture.
  const cropToggle = useMemo(
    () =>
      visiblePaneEntries.length > 0 ? (
        <RecordingCanvasTools
          activeTrack={activeVideoTrack}
          bakeCamera={bakeCamera}
          cameraPane={cameraPane}
          isEnabled={canEditActiveTrack}
          isFrameEnabled={canResizeActiveTrack}
          isRecenterEnabled={canRecenterPrimary}
          isSelectEnabled={visiblePaneEntries.length > 0}
          onCameraOverlayReset={onCameraOverlayChange}
          onChange={onRecordingOutputChange}
          onRecenterReset={recenter.reset}
          onToolChange={changeCanvasTool}
          outputs={recordingOutput}
          screenPane={screenPane}
          tool={canvasTool}
        />
      ) : undefined,
    [
      activeVideoTrack,
      bakeCamera,
      cameraPane,
      canEditActiveTrack,
      canResizeActiveTrack,
      canRecenterPrimary,
      canvasTool,
      changeCanvasTool,
      onCameraOverlayChange,
      onRecordingOutputChange,
      recenter.reset,
      recordingOutput,
      screenPane,
      visiblePaneEntries.length,
    ],
  );
  useEffect(() => {
    playhead.publish(0, 0);
  }, [artifactId, playhead]);

  // Arrows either move the selected layer or the playhead, never both: nudging
  // needs the selection tool, a movable layer and a parked playhead.
  const canNudgeActiveTrack =
    canvasTool === "select" && canMoveActiveVideoTrack && !isPlaying;
  useExportWindowShortcuts({
    onMoveBackward: canMoveActiveVideoTrack
      ? moveActiveVideoTrackBackward
      : undefined,
    onMoveForward: canMoveActiveVideoTrack
      ? moveActiveVideoTrackForward
      : undefined,
    onNudge: canNudgeActiveTrack ? nudgeActiveTrack : undefined,
    onRecenter: canRecenterPrimary ? toggleRecenterTool : undefined,
    onResizeCanvas: canResizeActiveTrack ? toggleCanvasTool : undefined,
    onSelectTool: hasVisiblePanes ? toggleSelectTool : undefined,
    onStep: !canNudgeActiveTrack && layout ? timelineBlade.step : undefined,
    onToggleCrop: hasVisiblePanes ? toggleCropTool : undefined,
    onTogglePlayback: layout ? togglePlayback : undefined,
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
  const changeSelectedTrack = useCallback(
    (trackId: RecordingTrackId) => {
      keyboardTimeline.selection.onClear();
      onSelectedTrackChange?.(trackId);
    },
    [keyboardTimeline.selection, onSelectedTrackChange],
  );
  // The copied frame must use the output on screen, which during a resize is
  // the draft; a ref keeps the handler stable without staling the payload.
  const copyPayloadRef = useRef({
    bakeCamera,
    cameraOverlay,
    cursorEffects,
    keyboardEffects,
    recordingOutput: effectiveRecordingOutput,
  });
  copyPayloadRef.current = {
    bakeCamera,
    cameraOverlay,
    cursorEffects,
    keyboardEffects,
    recordingOutput: effectiveRecordingOutput,
  };
  const getPositionMs = player.getPositionMs;
  const copyCurrentFrame = useCallback(() => {
    setCopyError(null);
    return copyRecordingPreviewFrameToClipboard({
      artifactId,
      bakeCamera: copyPayloadRef.current.bakeCamera,
      cameraOverlay: copyPayloadRef.current.cameraOverlay,
      cursorEffects: copyPayloadRef.current.cursorEffects,
      keyboardEffects: copyPayloadRef.current.keyboardEffects,
      positionMs: getPositionMs(),
      recordingOutput: copyPayloadRef.current.recordingOutput,
    }).catch((cause: unknown) => {
      setCopyError(cause instanceof Error ? cause.message : String(cause));
      // Rethrown so the copy button knows the press failed and skips its check.
      throw cause;
    });
  }, [artifactId, getPositionMs]);

  return (
    <div className="flex min-h-0 grow flex-col [--recording-inspector-width:clamp(270px,23vw,300px)]">
      <div className="grid min-h-0 grow grid-cols-[var(--recording-inspector-width)_minmax(0,1fr)]">
        {inspector}
        <section className="relative flex min-h-0 min-w-0 flex-col">
          <div
            aria-hidden
            className="pointer-events-none absolute inset-0 -z-10 bg-black/5 dark:bg-black/25"
            data-preview-backdrop
          />
          {visibleLayout && visibleLayout.panes.length > 0 ? (
            <PreviewToolbar
              onZoomChange={setZoomPercent}
              tools={cropToggle}
              zoomPercent={zoomPercent}
            />
          ) : null}
          <div className="flex min-h-0 grow items-stretch justify-center">
            {!layout ? (
              <div className="flex grow items-center justify-center gap-3 text-xs text-muted">
                <CircularProgressBar
                  aria-label="Preparing recording preview"
                  isIndeterminate
                  size={32}
                  strokeWidth={10}
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
                    // before it reaches the export window's state.
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
              <AudioVisualizer
                audioTracks={audioTracks}
                enabledTracks={enabledTracks}
                playhead={playhead}
                volumes={audioVolumeByStream}
              />
            )}
          </div>

          {audioError ? (
            <p className="m-0 px-4 pb-2 text-xs text-error">{audioError}</p>
          ) : null}
          {player.error ? (
            <p className="m-0 px-4 pb-2 text-xs text-error">{player.error}</p>
          ) : null}
          {copyError ? (
            <p className="m-0 px-4 pb-2 text-xs text-error">{copyError}</p>
          ) : null}

          {layout ? (
            <RecordingPlaybackControls
              durationMs={timelineBlade.timelineDurationMs}
              isPlaying={player.isPlaying}
              onCopyCurrentFrame={copyCurrentFrame}
              onPause={player.pause}
              onPlay={player.play}
              playhead={playhead}
            />
          ) : null}
        </section>
      </div>

      {layout ? (
        isPreparingAudio ? (
          <div className="flex h-24 shrink-0 items-center justify-center gap-2 border-t border-muted/15 text-xs text-muted">
            <CircularProgressBar
              aria-label="Preparing audio preview"
              isIndeterminate
              size={22}
              strokeWidth={8}
            />
            Preparing audio tracks
          </div>
        ) : (
          <RecordingTrackLanes
            adjustedKeyboardFragmentIds={keyboardTimeline.adjustedFragmentIds}
            audioTracks={audioTracks}
            blade={timelineBlade.blade}
            durationMs={timelineBlade.timelineDurationMs}
            enabledTracks={enabledTracks}
            enabledVideoTracks={selectedVideoTracks}
            hiddenKeyboardFragmentIds={keyboardTimeline.hiddenFragmentIds}
            hiddenKeyboardItemIds={keyboardTimeline.hiddenItemIds}
            keyboardItems={keyboardTimeline.items}
            keyboardSelection={keyboardTimeline.selection}
            layout={layout}
            onEnabledTracksChange={changeEnabledTracks}
            onEnabledVideoTracksChange={changeEnabledVideoTracks}
            onSeek={timelineBlade.seek}
            onSelectedTrackChange={changeSelectedTrack}
            onVideoTrackOrderChange={onVideoTrackOrderChange}
            playhead={playhead}
            selectedTrack={selectedTrack}
            sourceDurationMs={durationMs}
            thumbnails={timelineThumbnails}
            videoTrackOrder={videoTrackOrderList}
            volumes={audioVolumeByStream}
          />
        )
      ) : null}
    </div>
  );
}
