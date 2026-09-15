// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { CircularProgress } from "../../../components/base/circular-progress/circular-progress";
import { Text } from "../../../components/base/text/text";
import {
  cameraOverlayGeometry,
  uncroppedCameraPreviewOverlay,
} from "../camera-overlay-geometry";
import { scaledCameraOverlay } from "../camera-overlay-placement";
import { usePublishKeyboardShortcut } from "../keyboard-shortcut-channel";
import { useRecenterInsetControls } from "../recenter-inset-channel";
import {
  DEFAULT_KEYBOARD_EFFECTS,
  defaultCameraOverlay,
} from "../recording-export-settings";
import { createRecordingTimelineEdit } from "../recording-timeline-edit";
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
import { EditorToolId } from "../tool-panels/tool-registry";
import { useCanvasTool } from "../tool-panels/use-canvas-tool";
import { useToolPanelFollowsTool } from "../tool-panels/use-tool-panel-follows-tool";
import { CameraOverlaySettings, RecordingVideoTrackId } from "../types";
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
import { normalizedRecordingSelection } from "./recording-selection";
import { RecordingTrackLanes } from "./recording-track-lanes";
import { ResizableRecordingTimelineArea } from "./resizable-recording-timeline-area";
import { createPlayhead } from "./scrub-playhead";
import { useRecordingCanvasContextMenu } from "./use-recording-canvas-context-menu";
import {
  KEYBOARD_LAYER_ID,
  useRecordingKeyboardPreviewEditing,
} from "./use-recording-keyboard-canvas-editing";
import { useRecordingRecenter } from "./use-recording-recenter";
import { useRecordingSelectionNudge } from "./use-recording-selection-nudge";
import { useRecordingTimelineBlade } from "./use-recording-timeline-blade";
import { useRecordingToolbar } from "./use-recording-toolbar";
import { useRecordingTrackSelection } from "./use-recording-track-selection";
import { useRecordingTrimPreview } from "./use-recording-trim-preview";

import type { RecordingSelectionGestureEvent } from "../use-recording-preview-surface";
import type { ScrubPreviewProps } from "./scrub-preview";

const EMPTY_AUDIO_TRACKS: NonNullable<ScrubPreviewProps["audioTracks"]> = [];
const FRAME_LAYER_ID = 0xffffffff;
const AUTO_FIT_MOVE_EDGE = 1 << 17;
const AUTO_FIT_COMMIT_EDGE = 1 << 18;

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
  const selectionGestureRef = useRef<{
    cameraOverlaySnapshot: CameraOverlaySettings | null;
    lastDeltaX: number;
    lastDeltaY: number;
    lastScale: number;
    operation: RecordingSelectionGestureEvent["operation"];
    outputSnapshot: RecordingOutputSettings[RecordingVideoTrackId] | null;
    paneIndex: number;
    trackId: RecordingVideoTrackId;
  } | null>(null);
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
        canvasTool !== "arrow") ||
      !activeVideoTrack ||
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
          mode: canvasTool === "arrow" ? "select" : canvasTool,
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
      mode: canvasTool === "arrow" ? "select" : canvasTool,
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
  const selectionGesture = (event: RecordingSelectionGestureEvent) => {
    if (keyboardCanvas.applyGesture(event)) return;
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
    if (event.phase === "begin") {
      if (
        !trackId ||
        (isFrameGesture
          ? canvasTool !== "canvas"
          : isCropGesture
            ? canvasTool !== "crop"
            : canvasTool !== "select") ||
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
      // A baked overlay is placed in the screen output's own pixels, and the
      // native gesture reports its deltas as a share of that canvas.
      const canvas = screenshotOutputDimensions(
        effectiveRecordingOutput.primary,
      );
      const moveX = event.deltaX * canvas.width;
      const moveY = event.deltaY * canvas.height;
      const frameX = start.frameX + moveX;
      const frameY = start.frameY + moveY;
      let next: CameraOverlaySettings;
      if (event.operation === "cropMove") {
        next = {
          ...start,
          frameX,
          frameY,
        };
      } else if (event.operation === "cropResize") {
        let left = start.frameX;
        let top = start.frameY;
        let right = left + start.frameWidth;
        let bottom = top + start.frameHeight;
        if ((event.edges & 1) !== 0) left += moveX;
        if ((event.edges & 2) !== 0) right += moveX;
        if ((event.edges & 4) !== 0) top += moveY;
        if ((event.edges & 8) !== 0) bottom += moveY;
        next = {
          ...start,
          frameHeight: bottom - top,
          frameWidth: right - left,
          frameX: left,
          frameY: top,
        };
      } else if (event.operation === "radius") {
        next = {
          ...start,
          radiusPercent: Math.min(50, Math.max(0, event.scale)),
        };
      } else if (event.operation === "resize") {
        next = scaledCameraOverlay(
          start,
          Math.min(8, Math.max(0, event.scale)),
          { x: frameX, y: frameY },
        );
      } else {
        next = {
          ...start,
          cameraX: start.cameraX + moveX,
          cameraY: start.cameraY + moveY,
          frameX,
          frameY,
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
    // Native gesture deltas arrive as a share of the layer's own canvas.
    const canvas = screenshotOutputDimensions(snapshot);
    const moveX = event.deltaX * canvas.width;
    const moveY = event.deltaY * canvas.height;
    const cropX = snapshot.cropX + moveX;
    const cropY = snapshot.cropY + moveY;
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
      const workspace = {
        ...snapshot,
        items: [{ id: 0, output: snapshot }],
      };
      const resized = resizeScreenshotWorkspaceCanvasEdges({
        deltaX: event.deltaX,
        deltaY: event.deltaY,
        edges: event.edges,
        settings: workspace,
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
        cropHeight: snapshot.cropHeight * scale,
        cropWidth: snapshot.cropWidth * scale,
        cropX,
        cropY,
        imageWidth: snapshot.imageWidth * scale,
        imageX: transform(snapshot.imageX, snapshot.cropX, cropX),
        imageY: transform(snapshot.imageY, snapshot.cropY, cropY),
      };
    } else {
      next = {
        ...snapshot,
        cropX,
        cropY,
        imageX: snapshot.imageX + moveX,
        imageY: snapshot.imageY + moveY,
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
    onSelectionGesture: selectionGesture,
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
