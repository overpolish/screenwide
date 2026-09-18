// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { useRecenterInsetControls } from "../recenter-inset-channel";
import {
  scaledDimensions,
  scaledVideoDimensions,
  sourceScalePercent,
} from "../resolution";
import {
  screenshotWorkspaceItemOutput,
  ScreenshotOutputSettings,
} from "../screenshot-output";
import { EditorToolId } from "../tool-panels/tool-registry";
import { useCanvasTool } from "../tool-panels/use-canvas-tool";
import { useToolPanelFollowsTool } from "../tool-panels/use-tool-panel-follows-tool";
import { useEditorWindowShortcuts } from "../use-editor-window-shortcuts";
import { usePreviewZoom } from "../use-preview-zoom";

import {
  RecordingSectionProps,
  ScreenshotSectionProps,
} from "./editor-preview-section-props";
import { useProvideEditorToolbarTools } from "./editor-toolbar-context";
import { PreviewViewport } from "./preview-viewport";
import {
  deleteScreenshotLayer,
  moveScreenshotLayer,
} from "./screenshot-layer-actions";
import { ScreenshotStatusBar } from "./screenshot-status-bar";
import { ScreenshotTool, useScreenshotTools } from "./screenshot-tools";
import { ScrubPreview } from "./scrub-preview";
import { useScreenshotAnnotations } from "./use-screenshot-annotations";
import { useScreenshotRecenter } from "./use-screenshot-recenter";
import { useScreenshotTool } from "./use-screenshot-tool";
import { useToolFollowsAnnotation } from "./use-tool-follows-annotation";

/** The toolbar's own name for a tool, in the registry's vocabulary. */
const screenshotToolId = (tool: ScreenshotTool): EditorToolId | null =>
  tool === "canvas" ? "frame" : tool;

/**
 * The screenshot section. Sibling to `RecordingSection`, and the reason the
 * frame around them does not know what it is showing.
 */
export function ScreenshotSection({
  artifact,
  isExportOpen = false,
  isSaving = false,
  onBackgroundRadiusChange,
  onBackgroundRadiusChangeEnd,
  onCanvasResize,
  onOutputChange,
  onRadiusChangeEnd,
  onSelectedItemChange,
  screenshotOutput,
  selectedItemId = null,
}: ScreenshotSectionProps) {
  const { reportZoom, requestZoom, zoomPercent, zoomRequest } =
    usePreviewZoom();
  const [activeTool, setActiveTool] = useCanvasTool<
    Exclude<ScreenshotTool, null>
  >("screenshot", "select");
  const tool = activeTool;
  const selectedItem = artifact.items.find(
    (item) => item.id === selectedItemId,
  );
  const selectedOutput =
    screenshotOutput && selectedItem
      ? screenshotWorkspaceItemOutput(screenshotOutput, selectedItem.id)
      : null;
  // Which arrow the native handles are on, which one the halo is under, and
  // the edits that reach them.
  const annotations = useScreenshotAnnotations({
    onOutputChange,
    selectedItemId,
    selectedOutput,
  });
  // The panel follows the tool in hand, so choosing one is all a button or a
  // shortcut has to do - except while an arrow is chosen, whose own panel
  // stands in front of the tool's until it is let go of.
  useToolPanelFollowsTool(
    "screenshot",
    screenshotToolId(tool),
    annotations.hasSelection,
  );
  const setTool = useScreenshotTool({
    clearSelection: annotations.clearSelection,
    selectedKind: annotations.selectedKind,
    setActiveTool,
    tool,
  });
  // Both drawing tools pick up either shape, so the tool follows the annotation
  // that was chosen with it: the panel and the next press never disagree.
  useToolFollowsAnnotation(annotations.selectedKind, tool, setTool);
  const newestItemId = artifact.items[artifact.items.length - 1]?.id ?? null;
  const moveSelectedLayer = (
    direction: "backward" | "forward",
    itemId = selectedItemId,
  ) => {
    if (!screenshotOutput || itemId === null) return;
    const next = moveScreenshotLayer({
      direction,
      itemId,
      settings: screenshotOutput,
    });
    if (next !== screenshotOutput) onCanvasResize?.(next);
  };
  const deleteSelectedLayer = (itemId = selectedItemId) => {
    if (
      !screenshotOutput ||
      itemId === null ||
      screenshotOutput.items.length <= 1
    )
      return;
    const result = deleteScreenshotLayer({
      itemId,
      settings: screenshotOutput,
    });
    if (!result) return;
    onCanvasResize?.(result.settings);
    onSelectedItemChange?.(result.nextSelectedItemId);
  };
  const recenter = useScreenshotRecenter({
    artifactId: artifact.id,
    onOutputChange,
    selectedItem,
    selectedOutput,
  });
  // A keyboard nudge writes exactly the fields a select-tool drag writes - the
  // `move` branch of `selectionGesture` in preview-viewport.tsx. It is not
  // wrapped in an edit gesture on purpose: the history hook already groups
  // same-key edits that land within its grouping delay, so a held arrow folds
  // into one undo step while a pause starts the next. No snapping: the arrows
  // are the way to place a layer precisely.
  const nudgeRef = useRef<{
    after: ScreenshotOutputSettings;
    beforeX: number;
    beforeY: number;
    itemId: number;
  } | null>(null);
  const nudgeSelectedLayer = (
    directionX: number,
    directionY: number,
    coarse: boolean,
  ) => {
    if (!selectedItem || !selectedOutput) return;
    const pixels = coarse ? 10 : 1;
    const deltaX = directionX * pixels;
    const deltaY = directionY * pixels;
    // Key repeat can outrun React. While the committed settings have not come
    // back yet the props still hold the previous press's starting point, so
    // continue from what that press sent instead of repeating it.
    const pending = nudgeRef.current;
    const base =
      pending &&
      pending.itemId === selectedItem.id &&
      pending.beforeX === selectedOutput.cropX &&
      pending.beforeY === selectedOutput.cropY
        ? pending.after
        : selectedOutput;
    const next = {
      ...base,
      cropX: base.cropX + deltaX,
      cropY: base.cropY + deltaY,
      imageX: base.imageX + deltaX,
      imageY: base.imageY + deltaY,
    };
    nudgeRef.current = {
      after: next,
      beforeX: selectedOutput.cropX,
      beforeY: selectedOutput.cropY,
      itemId: selectedItem.id,
    };
    onOutputChange?.(next, selectedItem.id);
  };
  const tools = useScreenshotTools({
    newestItemId,
    onSelectedItemChange,
    selectedItemId,
    setTool,
    tool,
  });
  // The tools are drawn in the title bar, the way a unified toolbar carries
  // them, while their behaviour stays here with the picture they act on.
  useProvideEditorToolbarTools(tools);
  // The crop tool is the one tool you are "in": Enter accepts what is framed
  // and Escape backs out of it, and both simply put the tool down - the crop
  // itself was committed as each handle was released.
  const isCropping = tool === "crop";
  // A drawing tool is the other kind you are "in": Escape puts it down.
  const isAnnotating = tool === "arrow" || tool === "counter";
  // Every one of them hit-tests the annotations on the layer; only a drawing
  // tool makes a new one, and only it takes every press over the picture.
  const annotationTool =
    tool === "arrow" || tool === "counter" || tool === "select"
      ? tool
      : undefined;
  const hasSelectedAnnotation = annotations.hasSelection;
  const leaveCropTool = () => {
    setTool((current) => (current === "crop" ? null : current));
  };
  // Escape backs out one layer at a time: an arrow in hand is let go first,
  // and only a second press puts the tool itself down.
  const leaveModalTool = () => {
    if (hasSelectedAnnotation) {
      annotations.clearSelection();
      return;
    }
    setTool((current) =>
      current === "arrow" || current === "counter" || current === "crop"
        ? null
        : current,
    );
  };
  // The Select panel's padding controls reach this workspace's analysis
  // through here, alongside the refresh a crop drag ends with.
  useRecenterInsetControls("screenshot", recenter);
  useEditorWindowShortcuts({
    // The arrow is drawn on a layer, so the key takes one in hand the way the
    // crop key does.
    onArrowTool: () => {
      if (selectedItemId === null) onSelectedItemChange?.(newestItemId);
      setTool((current) => (current === "arrow" ? null : "arrow"));
    },
    onConfirm: isCropping ? leaveCropTool : undefined,
    // A counter is dropped on a layer, so its key takes one in hand too.
    onCounterTool: () => {
      if (selectedItemId === null) onSelectedItemChange?.(newestItemId);
      setTool((current) => (current === "counter" ? null : "counter"));
    },
    // Backspace and Delete take away the annotation the hand is pointing at
    // first, and only the layer when there is no annotation under them.
    onDelete: () => {
      if (!annotations.deleteTargeted()) deleteSelectedLayer();
    },
    onDeselect:
      isCropping || isAnnotating || hasSelectedAnnotation
        ? leaveModalTool
        : undefined,
    onMoveBackward: () => {
      moveSelectedLayer("backward");
    },
    onMoveForward: () => {
      moveSelectedLayer("forward");
    },
    onNudge:
      tool === "select" && selectedItem && selectedOutput
        ? nudgeSelectedLayer
        : undefined,
    onResizeCanvas: () => {
      setTool((current) => (current === "canvas" ? null : "canvas"));
    },
    onSelectTool: () => {
      setTool((current) => (current === "select" ? null : "select"));
    },
    onToggleCrop: () => {
      if (selectedItemId === null) onSelectedItemChange?.(newestItemId);
      setTool((current) => (current === "crop" ? null : "crop"));
    },
    ownsEscape: isCropping || isAnnotating || hasSelectedAnnotation,
  });

  return (
    <div className="flex min-h-0 min-w-0 grow flex-col">
      <PreviewViewport
        alt="Screenshot preview"
        annotationTool={annotationTool}
        artifactId={artifact.id}
        isEditing={tool === "crop"}
        isExportOpen={isExportOpen}
        isResizingCanvas={tool === "canvas"}
        isSaving={isSaving}
        isSelecting={tool === "select"}
        items={artifact.items}
        naturalHeight={artifact.height}
        naturalWidth={artifact.width}
        onAnnotationHover={annotations.onHoverChange}
        onBackgroundRadiusChange={onBackgroundRadiusChange}
        onBackgroundRadiusChangeEnd={onBackgroundRadiusChangeEnd}
        onCanvasResize={onCanvasResize}
        onCropChangeEnd={recenter.refresh}
        onItemSelect={onSelectedItemChange}
        onOutputChange={onOutputChange}
        onRadiusChangeEnd={onRadiusChangeEnd}
        onSelectedAnnotationChange={annotations.onSelectedChange}
        onZoomChange={reportZoom}
        screenshotOutput={screenshotOutput}
        selectedAnnotationId={annotations.selectedId}
        selectedItemId={selectedItemId}
        zoomRequest={zoomRequest}
      />
      <ScreenshotStatusBar
        onZoomChange={requestZoom}
        zoomPercent={zoomPercent}
      />
    </div>
  );
}

/**
 * The recording section: a preview you skim, with what the file is underneath.
 *
 * Framed exactly like the still beside it - no box, no border, just the
 * picture and its shadow - because they are the same kind of thing to the
 * person deciding whether to keep it.
 */
export function RecordingSection({
  artifact,
  audioTrackVolumes,
  bakeCamera,
  cameraOverlay,
  cameraResolutionScalePercent,
  cursorEffects,
  enabledStreamIndices,
  enabledVideoTracks,
  hasCursorData,
  hasKeyboardData,
  isExportOpen,
  isPreparingRecordingAudio,
  isPreparingRecordingPreview,
  isSaving,
  keyboardEffects,
  onCameraOverlayChange,
  onEnabledTracksChange,
  onEnabledVideoTracksChange,
  onKeyboardEffectsChange,
  onRecordingOutputChange,
  onRecordingTimelineEditChange,
  onSelectedTrackChange,
  onVideoTrackOrderChange,
  recordingOutput,
  recordingPreviewError,
  recordingPreviewLayout,
  recordingPreviewTracks,
  recordingTimelineEdit,
  resolutionScalePercent,
  selectedTrack,
}: RecordingSectionProps) {
  const primaryOutputDimensions = recordingOutput
    ? {
        height: recordingOutput.primary.height,
        width: recordingOutput.primary.width,
      }
    : scaledDimensions(
        artifact,
        resolutionScalePercent ?? sourceScalePercent(artifact),
      );
  const cameraOutputDimensions = recordingOutput
    ? {
        height: recordingOutput.camera.height,
        width: recordingOutput.camera.width,
      }
    : artifact.camera
      ? scaledVideoDimensions({
          height: artifact.camera.height,
          scale: cameraResolutionScalePercent ?? 100,
          sourceScale: 100,
          width: artifact.camera.width,
        })
      : undefined;

  return (
    <div className="flex min-h-0 grow flex-col">
      <ScrubPreview
        artifactId={artifact.id}
        audioError={recordingPreviewError}
        audioTracks={recordingPreviewTracks}
        audioTrackVolumes={audioTrackVolumes}
        bakeCamera={bakeCamera}
        cameraOverlay={cameraOverlay}
        cursorEffects={cursorEffects}
        durationMs={artifact.durationMs}
        enabledStreamIndices={enabledStreamIndices}
        enabledVideoTracks={enabledVideoTracks}
        hasCursorData={hasCursorData}
        hasKeyboardData={hasKeyboardData}
        isExportOpen={isExportOpen}
        isPreparingAudio={isPreparingRecordingAudio}
        isPreparingPreview={isPreparingRecordingPreview}
        isSaving={isSaving}
        key={artifact.id}
        keyboardEffects={keyboardEffects}
        keyboardMaximumWidthUnits={artifact.keyboardMaximumWidthUnits}
        onCameraOverlayChange={onCameraOverlayChange}
        onEnabledTracksChange={onEnabledTracksChange}
        onEnabledVideoTracksChange={onEnabledVideoTracksChange}
        onKeyboardEffectsChange={onKeyboardEffectsChange}
        onRecordingOutputChange={onRecordingOutputChange}
        onRecordingTimelineEditChange={onRecordingTimelineEditChange}
        onSelectedTrackChange={onSelectedTrackChange}
        onVideoTrackOrderChange={onVideoTrackOrderChange}
        previewLayout={recordingPreviewLayout}
        previewOutputDimensions={{
          primary: primaryOutputDimensions,
          ...(cameraOutputDimensions ? { camera: cameraOutputDimensions } : {}),
        }}
        previewSourceDimensions={{
          primary: { height: artifact.height, width: artifact.width },
          ...(artifact.camera
            ? {
                camera: {
                  height: artifact.camera.height,
                  width: artifact.camera.width,
                },
              }
            : {}),
        }}
        recordingOutput={recordingOutput}
        recordingTimelineEdit={recordingTimelineEdit}
        selectedTrack={selectedTrack}
      />
    </div>
  );
}
