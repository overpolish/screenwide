// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { t } from "../../../i18n/i18n";
import { useAnnotationDraft } from "../annotations/annotation-draft";
import { shownTool } from "../choice/editor-choice";
import { useSelectHold } from "../choice/use-select-hold";
import { useToolFollowsChoice } from "../choice/use-tool-follows-choice";
import { placeScreenshotImage } from "../images/image-api";
import { useIncomingImages } from "../images/use-incoming-images";
import { PreviewViewport } from "../preview/preview-viewport";
import {
  screenshotBetween,
  useTweenedOutput,
} from "../preview/use-tweened-output";
import { useRecenterInsetControls } from "../recenter-inset-channel";
import { screenshotChoices } from "../screenshot/screenshot-choices";
import {
  columnMode,
  columnTool,
  screenshotIsland,
  ScreenshotTool,
} from "../screenshot/screenshot-column";
import { screenshotWorkspaceItemOutput } from "../screenshot/screenshot-output";
import { useScreenshotAnnotations } from "../screenshot/use-screenshot-annotations";
import { useScreenshotLayerActions } from "../screenshot/use-screenshot-layer-actions";
import { useScreenshotLibrary } from "../screenshot/use-screenshot-library";
import { useScreenshotNudge } from "../screenshot/use-screenshot-nudge";
import { useScreenshotRecenter } from "../screenshot/use-screenshot-recenter";
import { useScreenshotTool } from "../screenshot/use-screenshot-tool";
import { useEditorWindowShortcuts } from "../shortcuts/use-editor-window-shortcuts";
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
  // ⌘ (Ctrl) held over a pen or a chosen layer holds Select (`canHoldSelect`).
  const hold = useSelectHold({
    hasLayerInHand: selectedItemId !== null,
    isExportOpen,
    onTook: () => {
      setActiveTool("select");
    },
    tool,
  });
  const pictureTool = hold.isHeld ? "select" : tool;
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
    // Deleting the annotations chosen with Select leaves nothing chosen:
    // their layer goes with them rather than opening the Screenshot mode.
    onChoiceDeleted: () => {
      if (tool === "select") onSelectedItemChange?.(null);
    },
    onOutputChange,
    onSelectedItemChange,
    onWorkspaceChange: onCanvasResize,
    selectedItemId,
    selectedOutput,
    sourceWidths: new Map(artifact.items.map((item) => [item.id, item.width])),
    workspace: screenshotOutput,
  });
  // The Screenshot mode is Select on a chosen layer, however it was chosen
  // (`columnMode`), with no annotations chosen on it: putting the layer down
  // or taking up another tool puts it away, and annotations chosen with
  // Select's band show their own island instead.
  const mode = columnMode(
    tool,
    selectedItem !== undefined && !annotations.hasSelection,
  );
  // The pen a click on an annotation calls for shows at once, though it is
  // taken up only as the press ends, so the column never passes through
  // Select on the way to it.
  const shown = shownTool({
    isHeld: hold.isHeld,
    kind: annotations.selectedKind,
    tool,
  });
  const isScreenOpen = mode === "screen";
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
  // One annotation chosen takes up its pen once the gesture has ended.
  useToolFollowsChoice({
    ...hold,
    kind: annotations.selectedKind,
    setTool,
    tool,
  });
  const island = screenshotIsland({
    annotationCount: annotations.selectedIds.size,
    hasSelectedLayer: selectedItem !== undefined,
    selectedKind: annotations.selectedKind,
    tool,
  });
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
  // makes a new one, and only it takes every press over the picture.
  const annotationTool = isAnnotationTool(pictureTool)
    ? pictureTool
    : undefined;
  const hasSelectedAnnotation = annotations.hasSelection;
  const { backToSelect, chooseMode, takeUp, toggle } = screenshotChoices({
    clearAnnotations: annotations.clearSelection,
    newestItemId,
    onSelectedItemChange,
    selectedItemId,
    setTool,
    tool,
  });
  const library = useScreenshotLibrary({ backToSelect, island });
  usePublishEditorModes({
    kind: "screenshot",
    library,
    mode,
    onModeChange: chooseMode,
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
    onToggleScreenPanel: () => {
      chooseMode(isScreenOpen ? null : "screen");
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
          tool={hold.isHeld ? "select" : columnTool(shown)}
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
          isSelectHeld={hold.isHeld}
          isSelecting={pictureTool === "select"}
          items={artifact.items}
          naturalHeight={artifact.height}
          naturalWidth={artifact.width}
          onAnnotationHover={annotations.onHoverChange}
          onCanvasResize={onCanvasResize}
          onCropChangeEnd={recenter.refresh}
          onItemSelect={(itemId) => {
            // A layer chosen on the picture, or nothing, is the whole choice:
            // the annotations go in the same step, so the layer never shows
            // chosen beside them.
            annotations.clearSelection();
            onSelectedItemChange?.(itemId);
          }}
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
