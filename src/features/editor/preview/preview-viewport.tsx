// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

import { useAnnotationDefaults } from "../annotations/annotation-defaults";
import { isTurned, withShareOfTurn } from "../layer-rotation";
import {
  croppingScreenshotWorkspace,
  isCropOperation,
} from "../screenshot/screenshot-crop";
import { SourceRect } from "../screenshot/screenshot-geometry";
import { gesturedScreenshotLayer } from "../screenshot/screenshot-layer-gesture";
import {
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
  fitScreenshotWorkspaceToItems,
  resizeScreenshotWorkspaceCanvasEdges,
  screenshotLayout,
  screenshotWorkspaceItemOutput,
  screenshotOutputDimensions,
} from "../screenshot/screenshot-output";
import { screenshotAnnotationOutput } from "../screenshot/screenshot-preview-events";
import {
  normalizedScreenshotSelection,
  screenshotLayerTurn,
  screenshotSelectionTargets,
} from "../screenshot/screenshot-selection";
import {
  ScreenshotAnnotationChangeEvent,
  ScreenshotSelectionGestureEvent,
  useScreenshotPreviewSurface,
} from "../screenshot/use-screenshot-preview-surface";
import { drawingToolKind } from "../tool-panels/tool-registry";
import {
  endGestureAfter,
  useEditorEditGesture,
} from "../use-editor-edit-history";

import { PreviewPaneFit } from "./preview-transform";
import { useCropTurn } from "./use-crop-turn";

type PreviewViewportProps = {
  alt: string;
  artifactId: number;
  items: { height: number; id: number; width: number }[];
  naturalHeight: number;
  naturalWidth: number;
  /** The annotation tool in hand. "select" hit-tests the annotations already on
   * the layer; "arrow" and "counter" also make a new one on empty picture,
   * and the layer's own chrome stands down for as long as one is held. */
  annotationTool?: import("../annotations/annotation-defaults").AnnotationTool;
  /** The crop in hand keeps its shape as its handles are dragged. */
  cropKeepsShape?: boolean;
  isEditing?: boolean;
  isResizingCanvas?: boolean;
  isSaving?: boolean;
  isSelecting?: boolean;
  /** An export or confirm sheet is up: native input stands down under it. */
  isSheetOpen?: boolean;
  onAnnotationHover?: (annotationId: string | null) => void;
  onCanvasResize?: (settings: ScreenshotWorkspaceOutputSettings) => void;
  onCropChangeEnd?: (crop: SourceRect) => Promise<void>;
  onItemSelect?: (itemId: number | null) => void;
  onOutputChange?: (
    settings: ScreenshotOutputSettings,
    itemId?: number,
  ) => void;
  onPaneFitChange?: (fit: PreviewPaneFit) => void;
  onSelectedAnnotationsChange?: (annotationIds: readonly string[]) => void;
  screenshotOutput?: ScreenshotWorkspaceOutputSettings;
  selectedAnnotationIds?: readonly string[];
  selectedItemId?: number | null;
};

const AUTO_FIT_MOVE_EDGE = 1 << 17;
const AUTO_FIT_COMMIT_EDGE = 1 << 18;

export function PreviewViewport({
  alt,
  annotationTool,
  artifactId,
  cropKeepsShape = false,
  isEditing = false,
  isResizingCanvas = false,
  isSaving = false,
  isSelecting = false,
  isSheetOpen = false,
  items,
  naturalHeight,
  naturalWidth,
  onAnnotationHover,
  onCanvasResize,
  onCropChangeEnd,
  onItemSelect,
  onOutputChange,
  onPaneFitChange,
  onSelectedAnnotationsChange,
  screenshotOutput,
  selectedAnnotationIds,
  selectedItemId = null,
}: PreviewViewportProps) {
  const nativeFrameRef = useRef<HTMLDivElement | null>(null);
  const selectionGestureRef = useRef<{
    /** The Begin deltas, which a crop draw uses as its anchor. */
    anchor: { x: number; y: number };
    autoFitCheckpointed: boolean;
    autoFitUsed: boolean;
    itemId: number;
    lastDeltaX: number;
    lastDeltaY: number;
    lastEdges: number;
    lastScale: number;
    operation: ScreenshotSelectionGestureEvent["operation"];
    paneIndex: number;
    snapshot: ScreenshotOutputSettings;
    workspaceSnapshot: ScreenshotWorkspaceOutputSettings;
    lastAutoFitOutput?: ScreenshotWorkspaceOutputSettings;
  } | null>(null);
  const frameGestureRef = useRef<{
    edges: number;
    lastDeltaX: number;
    lastDeltaY: number;
    snapshot: ScreenshotWorkspaceOutputSettings;
  } | null>(null);
  const editGesture = useEditorEditGesture();
  // A fresh annotation's dress travels with the layout the native tool draws
  // from, and it is the tool's own: a counter's disc and an arrow's stroke are
  // different measurements of different things.
  const annotationKind = drawingToolKind(annotationTool) ?? "arrow";
  const annotationDefaults = useAnnotationDefaults(annotationKind);
  const [canvasResizeDraft, setCanvasResizeDraft] =
    useState<ScreenshotWorkspaceOutputSettings | null>(null);
  const workspaceOutput =
    (isResizingCanvas ? canvasResizeDraft : null) ?? screenshotOutput;
  const orderedItems = workspaceOutput
    ? workspaceOutput.items
        .map((itemOutput) => items.find((item) => item.id === itemOutput.id))
        .filter((item): item is (typeof items)[number] => item !== undefined)
    : items;
  const output = workspaceOutput
    ? screenshotOutputDimensions(workspaceOutput)
    : { height: naturalHeight, width: naturalWidth };
  const selectedItemIndex =
    selectedItemId === null || !workspaceOutput
      ? -1
      : workspaceOutput.items.findIndex(
          (itemOutput) => itemOutput.id === selectedItemId,
        );
  const selectedItemOutput =
    selectedItemId !== null && workspaceOutput
      ? screenshotWorkspaceItemOutput(workspaceOutput, selectedItemId)
      : undefined;
  const selectedItem =
    selectedItemId === null
      ? undefined
      : items.find((item) => item.id === selectedItemId);
  // The crop tool works on the upright picture, so the layer's turn eases out
  // before its crop is shown and back in once the tool is put down; the
  // select chrome turns with it on the way.
  const cropTurn = useCropTurn(
    isEditing,
    isTurned(selectedItemOutput?.rotation),
  );
  const isCropShown = cropTurn.isFlat;
  const previewOutput =
    workspaceOutput &&
    croppingScreenshotWorkspace(workspaceOutput, selectedItem, cropTurn);
  const frameGesture = (event: ScreenshotSelectionGestureEvent) => {
    if (event.operation !== "frameResize") return false;
    if (event.phase === "begin") {
      if (!isResizingCanvas || !workspaceOutput) return true;
      frameGestureRef.current = {
        edges: event.edges,
        lastDeltaX: 0,
        lastDeltaY: 0,
        snapshot: workspaceOutput,
      };
      editGesture.beginGesture();
      return true;
    }
    const active = frameGestureRef.current;
    if (!active) return true;
    if (event.phase === "cancel") {
      onCanvasResize?.(active.snapshot);
      setCanvasResizeDraft(null);
      frameGestureRef.current = null;
      requestAnimationFrame(editGesture.endGesture);
      return true;
    }
    const differsFromLastUpdate =
      Math.abs(event.deltaX - active.lastDeltaX) > 1e-9 ||
      Math.abs(event.deltaY - active.lastDeltaY) > 1e-9 ||
      event.edges !== active.edges;
    if (event.phase === "update" || differsFromLastUpdate) {
      const next = resizeScreenshotWorkspaceCanvasEdges({
        deltaX: event.deltaX,
        deltaY: event.deltaY,
        edges: event.edges,
        settings: active.snapshot,
      });
      setCanvasResizeDraft(next);
      onCanvasResize?.(next);
    }
    active.edges = event.edges;
    active.lastDeltaX = event.deltaX;
    active.lastDeltaY = event.deltaY;
    if (event.phase === "end") {
      frameGestureRef.current = null;
      requestAnimationFrame(() => {
        setCanvasResizeDraft(null);
        editGesture.endGesture();
      });
    }
    return true;
  };
  const selectionGesture = (event: ScreenshotSelectionGestureEvent) => {
    if (frameGesture(event)) return;
    if (event.phase === "begin") {
      const itemOutput = workspaceOutput?.items[event.paneIndex];
      const cropGesture = isCropOperation(event.operation);
      if (
        (!isSelecting && !(isEditing && cropGesture)) ||
        !workspaceOutput ||
        !itemOutput
      )
        return;
      const snapshot = screenshotWorkspaceItemOutput(
        workspaceOutput,
        itemOutput.id,
      );
      selectionGestureRef.current = {
        anchor: { x: event.deltaX, y: event.deltaY },
        autoFitCheckpointed: false,
        autoFitUsed: false,
        itemId: itemOutput.id,
        lastDeltaX: 0,
        lastDeltaY: 0,
        lastEdges: event.edges,
        lastScale: event.scale,
        operation: event.operation,
        paneIndex: event.paneIndex,
        snapshot,
        workspaceSnapshot: workspaceOutput,
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
      if (active.autoFitUsed || active.autoFitCheckpointed)
        onCanvasResize?.(active.workspaceSnapshot);
      else onOutputChange?.(active.snapshot, active.itemId);
      selectionGestureRef.current = null;
      requestAnimationFrame(editGesture.endGesture);
      return;
    }
    const autoFitCommit =
      event.operation === "move" && (event.edges & AUTO_FIT_COMMIT_EDGE) !== 0;
    if (autoFitCommit && active.lastAutoFitOutput) {
      const committed = active.lastAutoFitOutput;
      const committedItem = screenshotWorkspaceItemOutput(
        committed,
        active.itemId,
      );
      active.autoFitCheckpointed = true;
      active.autoFitUsed = false;
      active.lastAutoFitOutput = undefined;
      active.lastDeltaX = 0;
      active.lastDeltaY = 0;
      active.lastEdges = event.edges;
      active.lastScale = event.scale;
      active.snapshot = committedItem;
      active.workspaceSnapshot = committed;
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
      ((event.operation === "resize" || event.operation === "cropResize") &&
        (Math.abs(event.scale - 1) > 1e-9 ||
          Math.abs(event.deltaX) > 1e-9 ||
          Math.abs(event.deltaY) > 1e-9)) ||
      (event.operation === "rotate" &&
        Math.abs(event.scale - active.lastScale) > 1e-9);
    const differsFromLastUpdate =
      Math.abs(event.deltaX - active.lastDeltaX) > 1e-9 ||
      Math.abs(event.deltaY - active.lastDeltaY) > 1e-9 ||
      event.edges !== active.lastEdges ||
      ((event.operation === "resize" ||
        event.operation === "rotate" ||
        event.operation === "cropResize") &&
        Math.abs(event.scale - active.lastScale) > 1e-9);
    const cropOperation = isCropOperation(event.operation);
    const shouldApply =
      event.phase === "end" ? cropOperation || differsFromLastUpdate : changed;
    const source = items.find((item) => item.id === active.itemId);
    if (!source) return;
    const next = gesturedScreenshotLayer({
      anchor: active.anchor,
      event,
      output,
      snapshot: active.snapshot,
      source,
      workspace: active.workspaceSnapshot,
    });
    if (shouldApply) {
      const autoFit =
        event.operation === "move" && (event.edges & AUTO_FIT_MOVE_EDGE) !== 0;
      if (autoFit) {
        const fitted = fitScreenshotWorkspaceToItems({
          initial: active.workspaceSnapshot,
          movedItemId: active.itemId,
          movedItemOutput: next,
        });
        active.autoFitUsed = true;
        active.lastAutoFitOutput = fitted.output;
        onCanvasResize?.(fitted.output);
      } else if (active.autoFitUsed && event.operation === "move") {
        active.autoFitUsed = false;
        onCanvasResize?.({
          ...active.workspaceSnapshot,
          items: active.workspaceSnapshot.items.map((item) =>
            item.id === active.itemId ? { ...item, output: next } : item,
          ),
        });
      } else {
        onOutputChange?.(next, active.itemId);
      }
    }
    active.lastEdges = event.edges;
    if (event.phase === "update") finaliseGestureFrame();
    if (event.phase === "end") {
      // The inset colour behind the new crop is read from the picture and
      // lands later; it belongs to the crop's undo step, not one of its own.
      const refreshed =
        (cropOperation && onCropChangeEnd?.(next.sourceCrop)) ||
        Promise.resolve();
      selectionGestureRef.current = null;
      endGestureAfter(editGesture, refreshed);
    }
    return;
  };
  // A finished arrow gesture is one edit, and so is a text box's typing,
  // whose commits arrive as it is typed so the panel can dress the box.
  const annotationChange = (event: ScreenshotAnnotationChangeEvent) => {
    if (event.textEdit === "begin") editGesture.beginGesture();
    onSelectedAnnotationsChange?.(event.selectedAnnotationIds);
    const changed = screenshotAnnotationOutput(workspaceOutput, event);
    if (changed) onOutputChange?.(changed.output, changed.itemId);
    if (event.textEdit === "end") requestAnimationFrame(editGesture.endGesture);
  };
  const isDrawingArrows = annotationTool != null && annotationTool !== "select";
  const selectionOverlay =
    isResizingCanvas && workspaceOutput
      ? {
          layerId: 0xffffffff,
          paneIndex: 0,
          radiusPercent: workspaceOutput.backgroundRadiusPercent,
          rect: { height: 1, width: 1, x: 0, y: 0 },
        }
      : (isSelecting || isEditing || isDrawingArrows) &&
          selectedItemIndex >= 0 &&
          selectedItem &&
          selectedItemOutput
        ? {
            cropKeepsShape: isCropShown && cropKeepsShape,
            cropMode: isCropShown,
            paneIndex: selectedItemIndex,
            radiusPercent: selectedItemOutput.radiusPercent,
            ...screenshotLayerTurn(
              withShareOfTurn(selectedItemOutput, cropTurn.share),
              isCropShown,
            ),
            ...normalizedScreenshotSelection(
              screenshotLayout(selectedItem, selectedItemOutput),
              output,
              isCropShown ? "crop" : "select",
            ),
          }
        : null;
  const selectionTargets =
    (isSelecting || isEditing || isDrawingArrows) && workspaceOutput
      ? screenshotSelectionTargets(items, workspaceOutput, {
          ...output,
          cropMode: isCropShown,
        })
      : null;
  useScreenshotPreviewSurface({
    annotationDefaults,
    annotationTool,
    artifactId,
    canvasRef: nativeFrameRef,
    interactionOutput: workspaceOutput,
    isEditorSuspended: isSaving || isSheetOpen,
    isEnabled: workspaceOutput !== undefined,
    onAnnotationChange: annotationChange,
    onAnnotationHover,
    onPaneFitChange,
    onSelectionChange: (paneIndex) => {
      // A click on nothing with Select in hand lets the layer go, and with it
      // the Screenshot mode it opened.
      if (paneIndex === null) {
        if (isSelecting && selectedItemId !== null) onItemSelect?.(null);
        return;
      }
      const itemOutput = workspaceOutput?.items[paneIndex];
      if (itemOutput) onItemSelect?.(itemOutput.id);
    },
    onSelectionGesture: selectionGesture,
    output: previewOutput,
    paneCount: orderedItems.length,
    selectedAnnotationIds,
    selection: selectionOverlay,
    selectionTargets,
    sourceKey: orderedItems
      .map((item) => item.id)
      .sort((first, second) => first - second)
      .join(":"),
  });
  return (
    <div
      aria-label={alt}
      className={`relative flex min-h-0 grow overflow-hidden rounded-[inherit] ${isDrawingArrows ? "cursor-crosshair" : isSelecting ? "cursor-move" : "cursor-grab"}`}
      data-recording-preview-viewport
      dir="ltr"
      ref={nativeFrameRef}
      role="img"
    >
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 -z-10 bg-black/5 dark:bg-black/25"
        data-preview-backdrop
      />
    </div>
  );
}
