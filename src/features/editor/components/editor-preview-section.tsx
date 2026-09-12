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
import { useScreenshotRecenter } from "./use-screenshot-recenter";

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
  // The panel follows the tool in hand, so choosing one is all a button or a
  // shortcut has to do.
  useToolPanelFollowsTool("screenshot", screenshotToolId(tool));
  const toolRef = useRef(tool);
  toolRef.current = tool;
  const setTool = (
    next: ScreenshotTool | ((current: ScreenshotTool) => ScreenshotTool),
  ) => {
    setActiveTool(typeof next === "function" ? next(toolRef.current) : next);
  };
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
  const selectedItem = artifact.items.find(
    (item) => item.id === selectedItemId,
  );
  const selectedOutput =
    screenshotOutput && selectedItem
      ? screenshotWorkspaceItemOutput(screenshotOutput, selectedItem.id)
      : null;
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
  const leaveCropTool = () => {
    setTool((current) => (current === "crop" ? null : current));
  };
  // The Select panel's padding controls reach this workspace's analysis
  // through here, alongside the refresh a crop drag ends with.
  useRecenterInsetControls("screenshot", recenter);
  useEditorWindowShortcuts({
    onConfirm: isCropping ? leaveCropTool : undefined,
    onDelete: deleteSelectedLayer,
    onDeselect: isCropping ? leaveCropTool : undefined,
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
    ownsEscape: isCropping,
  });

  return (
    <div className="flex min-h-0 min-w-0 grow flex-col">
      <PreviewViewport
        alt="Screenshot preview"
        artifactId={artifact.id}
        isEditing={tool === "crop"}
        isExportOpen={isExportOpen}
        isResizingCanvas={tool === "canvas"}
        isSaving={isSaving}
        isSelecting={tool === "select"}
        items={artifact.items}
        naturalHeight={artifact.height}
        naturalWidth={artifact.width}
        onBackgroundRadiusChange={onBackgroundRadiusChange}
        onBackgroundRadiusChangeEnd={onBackgroundRadiusChangeEnd}
        onCanvasResize={onCanvasResize}
        onCropChangeEnd={recenter.refresh}
        onItemSelect={onSelectedItemChange}
        onOutputChange={onOutputChange}
        onRadiusChangeEnd={onRadiusChangeEnd}
        onZoomChange={reportZoom}
        screenshotOutput={screenshotOutput}
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
