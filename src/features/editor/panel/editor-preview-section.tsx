// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { t } from "../../../i18n/i18n";
import { useAnnotationDraft } from "../annotations/annotation-draft";
import { useToolFollowsAnnotation } from "../annotations/use-tool-follows-annotation";
import { placeScreenshotImage } from "../images/image-api";
import { useIncomingImages } from "../images/use-incoming-images";
import { PreviewViewport } from "../preview/preview-viewport";
import { usePreviewZoom } from "../preview/use-preview-zoom";
import { useRecenterInsetControls } from "../recenter-inset-channel";
import {
  screenshotWorkspaceItemOutput,
  ScreenshotOutputSettings,
} from "../screenshot/screenshot-output";
import { ScreenshotStatusBar } from "../screenshot/screenshot-status-bar";
import {
  ScreenshotTool,
  useScreenshotTools,
} from "../screenshot/screenshot-tools";
import { useScreenshotAnnotations } from "../screenshot/use-screenshot-annotations";
import { useScreenshotLayerActions } from "../screenshot/use-screenshot-layer-actions";
import { useScreenshotRecenter } from "../screenshot/use-screenshot-recenter";
import { useScreenshotTool } from "../screenshot/use-screenshot-tool";
import { useEditorWindowShortcuts } from "../shortcuts/use-editor-window-shortcuts";
import {
  EditorToolId,
  drawingToolKind,
  isAnnotationTool,
} from "../tool-panels/tool-registry";
import { useCanvasTool } from "../tool-panels/use-canvas-tool";
import { useToolPanelFollowsTool } from "../tool-panels/use-tool-panel-follows-tool";

import { ScreenshotSectionProps } from "./editor-preview-section-props";
import { useProvideEditorToolbarTools } from "./editor-toolbar-context";

/** The toolbar's own name for a tool, in the registry's vocabulary. */
const screenshotToolId = (tool: ScreenshotTool): EditorToolId | null =>
  tool === "canvas" ? "frame" : tool;

/**
 * The screenshot section. Sibling to `RecordingSection`, and the reason the
 * frame around them does not know what it is showing.
 */
export function ScreenshotSection({
  artifact,
  isSaving = false,
  isSheetOpen = false,
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
    onSelectedItemChange,
    onWorkspaceChange: onCanvasResize,
    selectedItemId,
    selectedOutput,
    sourceWidths: new Map(artifact.items.map((item) => [item.id, item.width])),
    workspace: screenshotOutput,
  });
  // The panel follows the tool in hand, so choosing one is all a button or a
  // shortcut has to do - except while an arrow is chosen, whose own panel
  // stands in front of the tool's until it is let go of.
  useToolPanelFollowsTool(
    "screenshot",
    screenshotToolId(tool),
    annotations.hasSelection,
  );
  // What a drawing tool's panel shows while nothing is chosen: the dress its
  // next annotation is drawn in.
  useAnnotationDraft("screenshot", screenshotToolId(tool));
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
  const { deleteLayer: deleteSelectedLayer, moveLayer: moveSelectedLayer } =
    useScreenshotLayerActions({
      onCanvasResize,
      onSelectedItemChange,
      screenshotOutput,
      selectedItemId,
    });
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
  const isAnnotating = drawingToolKind(tool) !== null;
  // Every one of them hit-tests the annotations on the layer; only a drawing
  // tool makes a new one, and only it takes every press over the picture. The
  // marquee draws a band over the picture instead.
  const annotationTool = isAnnotationTool(tool) ? tool : undefined;
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
      drawingToolKind(current) !== null || current === "crop" ? null : current,
    );
  };
  // The Select panel's padding controls reach this workspace's analysis
  // through here, alongside the refresh a crop drag ends with.
  useRecenterInsetControls("screenshot", recenter);
  useEditorWindowShortcuts({
    // A chosen annotation moves through its layer's stacking; with none in
    // hand the layer itself moves.
    onArrange: (move) => {
      if (!annotations.arrangeSelected(move)) moveSelectedLayer(move);
    },
    onConfirm: isCropping ? leaveCropTool : undefined,
    // Backspace and Delete take away the annotation the hand is pointing at
    // first, and only the layer when there is no annotation under them.
    onDelete: () => {
      if (!annotations.deleteTargeted()) deleteSelectedLayer();
    },
    onDeselect:
      isCropping || isAnnotating || hasSelectedAnnotation
        ? leaveModalTool
        : undefined,
    onMarqueeTool: () => {
      if (selectedItemId === null) onSelectedItemChange?.(newestItemId);
      setTool((current) => (current === "marquee" ? null : "marquee"));
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
    // A drawing tool draws on a layer, so its key takes one in hand the way
    // the crop key does.
    onTool: (next) => {
      if (selectedItemId === null) onSelectedItemChange?.(newestItemId);
      setTool((current) => (current === next ? null : next));
    },
    ownsEscape: isCropping || isAnnotating || hasSelectedAnnotation,
  });
  useIncomingImages("screenshot", placeScreenshotImage, {
    // The image placed is chosen; a tool that shows it is put in hand,
    // unless the select tool or the image tool already is.
    onPlaced: () => {
      setTool((current) =>
        current === "select" || current === "image" ? current : "image",
      );
    },
  });

  return (
    <div className="flex min-h-0 min-w-0 grow flex-col">
      <PreviewViewport
        alt={t("editor-screenshot-preview")}
        annotationTool={annotationTool}
        artifactId={artifact.id}
        isEditing={tool === "crop"}
        isResizingCanvas={tool === "canvas"}
        isSaving={isSaving}
        isSelecting={tool === "select"}
        isSheetOpen={isSheetOpen}
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
        onSelectedAnnotationsChange={annotations.onSelectedIdsChange}
        onZoomChange={reportZoom}
        screenshotOutput={screenshotOutput}
        selectedAnnotationIds={[...annotations.selectedIds]}
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
