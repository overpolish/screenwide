// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { useAnnotationDraft } from "../../annotations/annotation-draft";
import { useSelectHold } from "../../choice/use-select-hold";
import { useToolFollowsChoice } from "../../choice/use-tool-follows-choice";
import { ChromeSetBack } from "../../panel/chrome-set-back";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
  defaultRecordingOutput,
} from "../../screenshot/screenshot-output";
import { RecordingStripBand } from "../../timeline/recording-strip/recording-strip-band";
import { createPlayhead } from "../../timeline/scrub-playhead";
import { useRecordingPreviewTracks } from "../../timeline/tracks/use-recording-preview-tracks";
import { usePublishEditorModes } from "../../title-bar/editor-title-modes";
import { EditorToolColumn } from "../../toolbar/editor-tool-column";
import { useEditorEditGesture } from "../../use-editor-edit-history";
import { useCopyRecordingFrame } from "../use-copy-recording-frame";
import { useProjectStill } from "../use-project-still";

import { CAPTIONS_LAYER_ID, OverlayLayerId } from "./overlay-layers";
import { RecordingCanvasTool } from "./recording-canvas-tool";
import { resolveScrubPreviewProps } from "./recording-preview-props";
import { RecordingPreviewStage } from "./recording-preview-stage";
import { useOfferedModes } from "./use-offered-modes";
import { useRecordingCanvasContextMenu } from "./use-recording-canvas-context-menu";
import { useRecordingColumn } from "./use-recording-column";
import { useRecordingIsland } from "./use-recording-island";
import { useRecordingPreviewCanvasTool } from "./use-recording-preview-canvas-tool";
import { useRecordingPreviewSelection } from "./use-recording-preview-selection";
import { useRecordingPreviewShortcuts } from "./use-recording-preview-shortcuts";
import { useRecordingPreviewTransport } from "./use-recording-preview-transport";
import {
  useRecordingSceneCanvas,
  useSceneModeChoice,
} from "./use-recording-scene-canvas";
import { useRecordingStripBand } from "./use-recording-strip-band";

import type { ScrubPreviewProps } from "./scrub-preview";

/** Editor playback whose decode, audio output and timeline are all owned by
 * Rust. */
export function NativeRecordingPreview(rawProps: ScrubPreviewProps) {
  const props = resolveScrubPreviewProps(rawProps);
  const {
    artifactId,
    bakeCamera,
    hasAnnotations,
    isExportOpen,
    onSelectedTrackChange,
    previewOutputDimensions,
    recordingOutput,
    selectedTrack,
  } = props;
  const screenCanvasRef = useRef<HTMLCanvasElement>(null);
  const cameraCanvasRef = useRef<HTMLCanvasElement>(null);
  const recenterRefreshRef = useRef<
    (crop: ScreenshotOutputSettings["sourceCrop"]) => Promise<void>
  >(() => Promise.resolve());
  const editGesture = useEditorEditGesture();
  // The strip under the picture, which knows what shows at the playhead,
  // sets it.
  const pressOverlayRef = useRef((_layerId: OverlayLayerId) => undefined);
  const [playhead] = useState(createPlayhead);
  const [canvasTool, setCanvasTool] = useState<RecordingCanvasTool>("select");
  const activeVideoTrack =
    selectedTrack === "primary" || selectedTrack === "camera"
      ? selectedTrack
      : null;
  // ⌘ (Ctrl) held over a pen or a chosen track holds Select (`canHoldSelect`).
  const hold = useSelectHold({
    hasLayerInHand: activeVideoTrack !== null,
    isExportOpen,
    onTook: () => {
      setCanvasTool("select");
    },
    tool: canvasTool,
  });
  const pictureTool = hold.isHeld ? "select" : canvasTool;
  // A canvas resize runs at pointer rate; committing every move to the export
  // window's state re-renders the inspector, lanes and timeline and starves
  // the native pane's layout loop. The gesture renders from this draft and
  // commits once on release, exactly like the screenshot editor.
  const [canvasResizeDraft, setCanvasResizeDraft] =
    useState<RecordingOutputSettings | null>(null);
  const [previewPositionMs, setPreviewPositionMs] = useState(0);
  const previewPlayingRef = useRef(false);
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
    composedVideoTracks,
    enabledTracks,
    selectedStreamIndices,
    selectedVideoTracks,
  } = useRecordingPreviewTracks(props);
  const canPreviewBakedCamera =
    bakeCamera &&
    composedVideoTracks.has("primary") &&
    composedVideoTracks.has("camera");
  const selection = useRecordingPreviewSelection(props, {
    activeVideoTrack,
    canPreviewBakedCamera,
    canvasTool: pictureTool,
    editGesture,
    effectiveRecordingOutput,
    previewPositionMs,
    recenterRefreshRef,
    selectedVideoTracks: composedVideoTracks,
    setCanvasResizeDraft,
  });
  const transport = useRecordingPreviewTransport(props, {
    activeVideoTrack,
    cameraCanvasRef,
    canvasTool: pictureTool,
    composedVideoTracks,
    effectiveRecordingOutput,
    isSelectHeld: hold.isHeld,
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
  });
  const {
    annotations,
    clearAnnotationRef,
    layout,
    player,
    visiblePaneEntries,
  } = transport;
  const sceneCanvas = useRecordingSceneCanvas({
    activeVideoTrack,
    otherSelectionMade:
      annotations.hasSelection ||
      selection.keyboardTimeline.selection.ids.size > 0 ||
      selection.captionSelection.ids.size > 0 ||
      transport.timelineBlade.blade.rangeSelection !== null ||
      transport.timelineBlade.blade.selectedSegmentId !== null,
    scenes: transport.scenes,
  });
  const canEditActiveTrack =
    activeVideoTrack !== null && composedVideoTracks.has(activeVideoTrack);
  const canResizeActiveTrack =
    canEditActiveTrack && (!bakeCamera || canPreviewBakedCamera);
  const hasVisiblePanes = visiblePaneEntries.length > 0;
  useRecordingCanvasContextMenu(
    {
      cameraPlacement: annotations.cameraPlacement,
      clips: annotations.clips,
      onClipsChange: annotations.onClipsChange,
      pinning: annotations.pinning,
      selectedIds: annotations.selectedIds,
    },
    sceneCanvas.paneMenu,
  );
  const { changeCanvasTool, isCropping } = useRecordingPreviewCanvasTool({
    activeVideoTrack,
    annotations,
    bakeCamera,
    canvasTool,
    clearAnnotationRef,
    clearOverlaySelection: selection.clearOverlaySelection,
    hasVisiblePanes,
    onSelectedTrackChange,
    setCanvasTool,
  });
  // One annotation chosen takes up its pen once the gesture has ended.
  useToolFollowsChoice({
    ...hold,
    kind: annotations.selectedKind,
    setTool: changeCanvasTool,
    tool: canvasTool,
  });
  const column = useRecordingColumn({
    activeVideoTrack,
    annotations,
    canvasTool,
    changeCanvasTool,
    isHeld: hold.isHeld,
    onSelectedTrackChange,
    picture: props.recorded.picture,
  });
  const { hasTrackChoice, kind, offered, parts } = useOfferedModes(
    props,
    column,
    { enabledTracks, selectedVideoTracks },
  );
  // What a pen's island shows while nothing is chosen: the dress its next
  // annotation is drawn in.
  useAnnotationDraft(
    "recording",
    canvasTool === "canvas" ? "frame" : canvasTool,
  );
  // The scenes chosen go with the Scene mode, and the mode with them.
  useSceneModeChoice({
    isSceneInHand: canvasTool === "scene",
    putAway: column.backToSelect,
    scenes: transport.scenes,
  });
  // A chosen overlay opens its mode.
  const openOverlayTool = (layerId: OverlayLayerId) => {
    column.openMode(layerId === CAPTIONS_LAYER_ID ? "captions" : "keyboard");
  };
  useEffect(() => {
    playhead.publish(0, 0);
  }, [artifactId, playhead]);
  useRecordingPreviewShortcuts({
    annotations,
    arrangeScenePane: sceneCanvas.arrangeSelectedPane,
    backToSelect: column.backToSelect,
    canEditActiveTrack,
    canResizeActiveTrack,
    canvasTool,
    changeCanvasTool,
    hasAnnotations,
    hasSceneSelection: transport.scenes.selection.ids.size > 0,
    hasVisiblePanes,
    isCropping,
    layout,
    nudgeActiveTrack: selection.nudgeActiveTrack,
    offered,
    ownsEscape: column.ownsEscape,
    player,
    step: transport.timelineBlade.step,
    toggleCrop: column.toggleCrop,
    toggleMode: column.toggleMode,
  });
  const band = useRecordingStripBand(props, {
    changeCanvasTool,
    openOverlayTool,
    playhead,
    pressOverlayRef,
    selection,
    transport,
  });
  const hasTracks = Boolean(layout) && (hasTrackChoice || band.hasSounds);
  const island = useRecordingIsland({ column, hasTracks, strip: band });
  usePublishEditorModes({
    kind,
    mode: column.mode,
    onModeChange: column.onModeChange,
    parts,
    tracks: hasTracks ? island.tracks : null,
  });
  const { copyError } = useCopyRecordingFrame(props, {
    annotationClips: annotations.clips,
    getPositionMs: player.getPositionMs,
    recordingOutput: effectiveRecordingOutput,
  });
  useProjectStill(props, {
    annotationClips: annotations.clips,
    hasPicture: composedVideoTracks.has("primary"),
  });
  return (
    <div className="flex min-h-0 grow flex-col gap-separate">
      <div className="flex min-h-0 min-w-0 grow gap-inline">
        <ChromeSetBack className="flex" isSetBack={isExportOpen}>
          <EditorToolColumn
            {...column.column}
            kind={kind}
            tool={hold.isHeld ? "select" : column.column.tool}
          />
        </ChromeSetBack>
        <RecordingPreviewStage
          {...props}
          {...transport}
          activeRecordingOutput={activeRecordingOutput}
          audioVolumeByStream={audioVolumeByStream}
          canPreviewBakedCamera={canPreviewBakedCamera}
          copyError={copyError}
          enabledTracks={enabledTracks}
          screenCanvasRef={screenCanvasRef}
        />
      </div>
      {layout ? (
        <RecordingStripBand
          isLocked={isExportOpen}
          strip={band.strip}
          transport={band.transport}
        />
      ) : null}
    </div>
  );
}
