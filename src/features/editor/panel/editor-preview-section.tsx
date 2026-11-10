// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { t } from "../../../i18n/i18n";
import { useAnnotationDraft } from "../annotations/annotation-draft";
import { useToolFollowsAnnotation } from "../annotations/use-tool-follows-annotation";
import { placeScreenshotImage } from "../images/image-api";
import { useIncomingImages } from "../images/use-incoming-images";
import { useEditorIslandWindow } from "../island/host/use-editor-island-window";
import { PreviewViewport } from "../preview/preview-viewport";
import {
  screenshotBetween,
  useTweenedOutput,
} from "../preview/use-tweened-output";
import { useRecenterInsetControls } from "../recenter-inset-channel";
import {
  columnMode,
  columnTool,
  screenshotIsland,
  ScreenshotTool,
} from "../screenshot/screenshot-column";
import { screenshotWorkspaceItemOutput } from "../screenshot/screenshot-output";
import { useScreenshotAnnotations } from "../screenshot/use-screenshot-annotations";
import { useScreenshotLayerActions } from "../screenshot/use-screenshot-layer-actions";
import { useScreenshotNudge } from "../screenshot/use-screenshot-nudge";
import { useScreenshotRecenter } from "../screenshot/use-screenshot-recenter";
import { useScreenshotTool } from "../screenshot/use-screenshot-tool";
import { useEditorWindowShortcuts } from "../shortcuts/use-editor-window-shortcuts";
import { useSelectHold } from "../shortcuts/use-select-hold";
import { usePublishEditorModes } from "../title-bar/editor-title-modes";
import { NO_CAPTURE_PARTS } from "../toolbar/editor-modes";
import { EditorToolColumn } from "../toolbar/editor-tool-column";
import {
  cropAspectKey,
  keepsShape,
  useCropAspectStore,
} from "../tools/crop/crop-aspect-store";
import { drawingToolKind, isAnnotationTool } from "../tools/tool-registry";

import { ChromeSetBack } from "./chrome-set-back";
import { ScreenshotSectionProps } from "./editor-preview-section-props";

/**
 * The screenshot section. Sibling to `RecordingSection`, and the reason the
 * frame around them does not know what it is showing.
 */
export function ScreenshotSection({
  artifact,
  isExportOpen = false,
  onCanvasResize,
  onOutputChange,
  onSelectedItemChange,
  screenshotOutput,
  selectedItemId = null,
}: ScreenshotSectionProps) {
  // Select is what is in hand when nothing else is: a tool put down, by its
  // button pressed again or by Escape, leaves Select.
  const [activeTool, setActiveTool] =
    useState<Exclude<ScreenshotTool, null>>("select");
  const tool = activeTool;
  // ⌘ (Ctrl) held over a pen puts Select on the picture until it is let go;
  // a press under it that takes hold of something puts Select in hand.
  const isSelectHeld = useSelectHold({
    canHold: drawingToolKind(tool) !== null && !isExportOpen,
    onTook: () => {
      setActiveTool("select");
    },
  });
  const pictureTool = isSelectHeld ? "select" : tool;
  const selectedItem = artifact.items.find(
    (item) => item.id === selectedItemId,
  );
  // The Screenshot mode is Select on a chosen layer, however it was chosen
  // (`columnMode`): putting the layer down or taking up another tool puts
  // it away.
  const mode = columnMode(tool, selectedItem !== undefined);
  const isScreenOpen = mode === "screen";
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
  // What a pen's island shows while nothing is chosen: the dress its next
  // annotation is drawn in.
  useAnnotationDraft("screenshot", drawingToolKind(tool));
  const setTool = useScreenshotTool({
    chooseSelection: annotations.onSelectedIdsChange,
    clearSelection: annotations.clearSelection,
    selectedIds: annotations.selectedIds,
    selectedKind: annotations.selectedKind,
    setActiveTool: (next) => {
      setActiveTool(next ?? "select");
    },
    tool,
  });
  // Choosing an annotation takes up its pen, whose island shows it.
  useToolFollowsAnnotation(annotations.selectedKind, tool, setTool);
  const island = screenshotIsland({
    annotationCount: annotations.selectedIds.size,
    hasSelectedLayer: selectedItem !== undefined,
    selectedKind: annotations.selectedKind,
    tool,
  });
  useEditorIslandWindow({ island, workspace: "screenshot" });
  const newestItemId =
    artifact.items.length > 0
      ? artifact.items[artifact.items.length - 1].id
      : null;
  const cropLayer = selectedItemId ?? newestItemId;
  const cropAspect = useCropAspectStore((state) =>
    cropLayer === null
      ? undefined
      : state.aspects[cropAspectKey("screenshot", cropLayer)],
  );
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
  // A setting chosen on an island is drawn as the picture moving to it.
  const drawnOutput = useTweenedOutput(
    screenshotOutput,
    { part: "picture", workspace: "screenshot" },
    screenshotBetween,
  );
  const nudgeSelectedLayer = useScreenshotNudge({
    onOutputChange,
    selectedItem,
    selectedOutput,
  });
  // The crop tool is the one tool you are "in": Enter accepts what is framed
  // and puts it down, the crop itself committed as each handle was released.
  const isCropping = tool === "crop";
  // Every one of these hit-tests the annotations on the layer; only a pen
  // makes a new one, and only it takes every press over the picture. The
  // marquee draws a band over the picture instead.
  const annotationTool = isAnnotationTool(pictureTool)
    ? pictureTool
    : undefined;
  const hasSelectedAnnotation = annotations.hasSelection;
  // Select first, then nothing chosen: taken up, Select would bring back the
  // annotations it held when it was put down (`useSelectToolMemory`). The
  // layer goes too, whether chosen by hand or by the tool that took one up,
  // since Select on a chosen layer is the Screenshot mode.
  const backToSelect = () => {
    setTool("select");
    annotations.clearSelection();
    onSelectedItemChange?.(null);
  };
  // A tool that draws on a layer takes one in hand, the newest where none
  // is chosen.
  const takeUp = (next: ScreenshotTool) => {
    if (next !== "select" && next !== "canvas" && selectedItemId === null)
      onSelectedItemChange?.(newestItemId);
    setTool(next);
  };
  const toggle = (next: ScreenshotTool) => {
    takeUp(tool === next ? "select" : next);
  };
  usePublishEditorModes({
    kind: "screenshot",
    mode,
    onModeChange: (next) => {
      if (next === null) backToSelect();
      else if (next === "frame") takeUp("canvas");
      else {
        // The Screenshot mode is Select on a layer, the newest where none
        // is chosen, with the annotations let go as any mode lets them go.
        if (selectedItemId === null) onSelectedItemChange?.(newestItemId);
        setTool("select");
        annotations.clearSelection();
      }
    },
    parts: NO_CAPTURE_PARTS,
    tracks: null,
  });
  // The Screenshot island's padding controls reach this workspace's analysis
  // through here, alongside the refresh a crop drag ends with.
  useRecenterInsetControls("screenshot", recenter);
  useEditorWindowShortcuts({
    // A chosen annotation moves through its layer's stacking; with none in
    // hand the layer itself moves.
    onArrange: (move) => {
      if (!annotations.arrangeSelected(move)) moveSelectedLayer(move);
    },
    onConfirm: isCropping ? backToSelect : undefined,
    // Backspace and Delete take away the annotation the hand is pointing at
    // first, and only the layer when there is no annotation under them.
    onDelete: () => {
      if (!annotations.deleteTargeted()) deleteSelectedLayer();
    },
    // Escape puts Select back in hand with nothing chosen.
    onDeselect:
      tool !== "select" || hasSelectedAnnotation || isScreenOpen
        ? backToSelect
        : undefined,
    // ⌘D lets go of everything chosen, the layer as well, with Select in
    // hand.
    onDeselectAll: backToSelect,
    onMarqueeTool: () => {
      toggle("marquee");
    },
    onNudge:
      tool === "select" && selectedItem && selectedOutput
        ? nudgeSelectedLayer
        : undefined,
    onResizeCanvas: () => {
      toggle("canvas");
    },
    onSelectTool: backToSelect,
    onToggleCrop: () => {
      toggle("crop");
    },
    onTool: toggle,
    ownsEscape: tool !== "select" || hasSelectedAnnotation || isScreenOpen,
  });
  useIncomingImages("screenshot", placeScreenshotImage, {
    // The image placed is chosen, and its pen taken up with it.
    onPlaced: () => {
      setTool("image");
    },
  });

  return (
    <div className="flex min-h-0 min-w-0 grow gap-inline">
      <ChromeSetBack className="flex" isSetBack={isExportOpen}>
        <EditorToolColumn
          isModeOpen={mode !== null}
          kind="screenshot"
          onToolChange={(next) => {
            if (next === "select") backToSelect();
            else takeUp(next);
          }}
          tool={isSelectHeld ? "select" : columnTool(tool)}
        />
      </ChromeSetBack>
      {/* The picture meets the tool column and the title bar on a rounded
          corner; the viewport takes it up, and the native drawables are
          clipped to it. */}
      <div className="flex min-h-0 min-w-0 grow flex-col rounded-ss-panel">
        <PreviewViewport
          alt={t("editor-screenshot-preview")}
          annotationTool={annotationTool}
          artifactId={artifact.id}
          cropKeepsShape={keepsShape(cropAspect)}
          isEditing={isCropping}
          isExportOpen={isExportOpen}
          isResizingCanvas={tool === "canvas"}
          isSelecting={pictureTool === "select"}
          items={artifact.items}
          naturalHeight={artifact.height}
          naturalWidth={artifact.width}
          onAnnotationHover={annotations.onHoverChange}
          onCanvasResize={onCanvasResize}
          onCropChangeEnd={recenter.refresh}
          onItemSelect={onSelectedItemChange}
          onOutputChange={onOutputChange}
          onSelectedAnnotationsChange={annotations.onSelectedIdsChange}
          screenshotOutput={drawnOutput}
          selectedAnnotationIds={[...annotations.selectedIds]}
          selectedItemId={selectedItemId}
        />
      </div>
    </div>
  );
}
