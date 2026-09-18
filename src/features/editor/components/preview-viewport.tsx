// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

import {
  useAnnotationAngleDefault,
  useAnnotationDefaults,
} from "../annotation-defaults";
import { PreviewZoomRequest } from "../preview-zoom-state";
import {
  applyScreenshotCropGesture,
  commitScreenshotCrop,
  uncroppedScreenshotPreviewOutput,
} from "../screenshot-crop";
import { SourceRect } from "../screenshot-geometry";
import {
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
  fitScreenshotWorkspaceToItems,
  resizeScreenshotWorkspaceCanvasEdges,
  screenshotLayout,
  screenshotWorkspaceItemOutput,
  screenshotOutputDimensions,
} from "../screenshot-output";
import { useEditorEditGesture } from "../use-editor-edit-history";
import {
  ScreenshotAnnotationChangeEvent,
  ScreenshotSelectionGestureEvent,
  useScreenshotPreviewSurface,
} from "../use-screenshot-preview-surface";

import { useRegisterPreviewFit } from "./preview-fit-context";
import { PreviewPaneFit } from "./preview-transform";
import {
  normalizedScreenshotSelection,
  screenshotSelectionTargets,
} from "./screenshot-selection";

type PreviewViewportProps = {
  alt: string;
  artifactId: number;
  items: { height: number; id: number; width: number }[];
  naturalHeight: number;
  naturalWidth: number;
  /** The annotation tool in hand. "select" hit-tests the annotations already on
   * the layer; "arrow" and "counter" also make a new one on empty picture,
   * and the layer's own chrome stands down for as long as one is held. */
  annotationTool?: "arrow" | "counter" | "select";
  isEditing?: boolean;
  /** Suspends native input, so the DOM over the viewport stays clickable. */
  isExportOpen?: boolean;
  isResizingCanvas?: boolean;
  isSaving?: boolean;
  isSelecting?: boolean;
  onAnnotationHover?: (annotationId: string | null) => void;
  onBackgroundRadiusChange?: (radiusPercent: number) => void;
  onBackgroundRadiusChangeEnd?: () => void;
  onCanvasResize?: (settings: ScreenshotWorkspaceOutputSettings) => void;
  onCropChangeEnd?: (crop: SourceRect) => void;
  onItemSelect?: (itemId: number) => void;
  onOutputChange?: (
    settings: ScreenshotOutputSettings,
    itemId?: number,
  ) => void;
  onPaneFitChange?: (fit: PreviewPaneFit) => void;
  onRadiusChangeEnd?: () => void;
  onSelectedAnnotationChange?: (annotationId: string | null) => void;
  onZoomChange?: (zoomPercent: number) => void;
  screenshotOutput?: ScreenshotWorkspaceOutputSettings;
  selectedAnnotationId?: string | null;
  selectedItemId?: number | null;
  zoomRequest?: PreviewZoomRequest;
};

const AUTO_FIT_MOVE_EDGE = 1 << 17;
const AUTO_FIT_COMMIT_EDGE = 1 << 18;

export function PreviewViewport({
  alt,
  annotationTool,
  artifactId,
  isEditing = false,
  isExportOpen = false,
  isResizingCanvas = false,
  isSaving = false,
  isSelecting = false,
  items,
  naturalHeight,
  naturalWidth,
  onAnnotationHover,
  onBackgroundRadiusChange,
  onBackgroundRadiusChangeEnd,
  onCanvasResize,
  onCropChangeEnd,
  onItemSelect,
  onOutputChange,
  onPaneFitChange,
  onRadiusChangeEnd,
  onSelectedAnnotationChange,
  onZoomChange,
  screenshotOutput,
  selectedAnnotationId = null,
  selectedItemId = null,
  zoomRequest,
}: PreviewViewportProps) {
  const nativeFrameRef = useRef<HTMLDivElement | null>(null);
  const selectionGestureRef = useRef<{
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
    lastScale: number;
    operation: "frameRadius" | "frameResize";
    snapshot: ScreenshotWorkspaceOutputSettings;
  } | null>(null);
  const editGesture = useEditorEditGesture();
  // A fresh annotation's dress travels with the layout the native tool draws
  // from, and it is the tool's own: a counter's disc and an arrow's stroke are
  // different measurements of different things.
  const annotationDefaults = useAnnotationDefaults(
    annotationTool === "counter" ? "counter" : "arrow",
  );
  const annotationCounterAngle = useAnnotationAngleDefault();
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
  const previewOutput =
    workspaceOutput && isEditing && selectedItemId !== null
      ? {
          ...workspaceOutput,
          items: workspaceOutput.items.map((itemOutput) => {
            const item = items.find(
              (candidate) => candidate.id === itemOutput.id,
            );
            return item && item.id === selectedItemId
              ? {
                  ...itemOutput,
                  output: uncroppedScreenshotPreviewOutput(
                    item,
                    itemOutput.output,
                  ),
                }
              : itemOutput;
          }),
        }
      : workspaceOutput;
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
  const frameGesture = (event: ScreenshotSelectionGestureEvent) => {
    if (event.operation !== "frameResize" && event.operation !== "frameRadius")
      return false;
    if (event.phase === "begin") {
      if (!isResizingCanvas || !workspaceOutput) return true;
      frameGestureRef.current = {
        edges: event.edges,
        lastDeltaX: 0,
        lastDeltaY: 0,
        lastScale: event.scale,
        operation: event.operation,
        snapshot: workspaceOutput,
      };
      editGesture.beginGesture();
      return true;
    }
    const active = frameGestureRef.current;
    if (!active || active.operation !== event.operation) return true;
    if (event.phase === "cancel") {
      if (active.operation === "frameRadius") {
        onBackgroundRadiusChange?.(active.snapshot.backgroundRadiusPercent);
        onBackgroundRadiusChangeEnd?.();
      } else {
        onCanvasResize?.(active.snapshot);
        setCanvasResizeDraft(null);
      }
      frameGestureRef.current = null;
      requestAnimationFrame(editGesture.endGesture);
      return true;
    }
    const differsFromLastUpdate =
      Math.abs(event.deltaX - active.lastDeltaX) > 1e-9 ||
      Math.abs(event.deltaY - active.lastDeltaY) > 1e-9 ||
      Math.abs(event.scale - active.lastScale) > 1e-9 ||
      event.edges !== active.edges;
    if (event.phase === "update" || differsFromLastUpdate) {
      if (active.operation === "frameRadius") {
        onBackgroundRadiusChange?.(Math.min(50, Math.max(0, event.scale)));
      } else {
        const next = resizeScreenshotWorkspaceCanvasEdges({
          deltaX: event.deltaX,
          deltaY: event.deltaY,
          edges: event.edges,
          settings: active.snapshot,
        });
        setCanvasResizeDraft(next);
        onCanvasResize?.(next);
      }
    }
    active.edges = event.edges;
    active.lastDeltaX = event.deltaX;
    active.lastDeltaY = event.deltaY;
    active.lastScale = event.scale;
    if (event.phase === "end") {
      frameGestureRef.current = null;
      requestAnimationFrame(() => {
        setCanvasResizeDraft(null);
        editGesture.endGesture();
      });
      if (active.operation === "frameRadius") onBackgroundRadiusChangeEnd?.();
    }
    return true;
  };
  const selectionGesture = (event: ScreenshotSelectionGestureEvent) => {
    if (frameGesture(event)) return;
    if (event.phase === "begin") {
      const itemOutput = workspaceOutput?.items[event.paneIndex];
      const cropGesture =
        event.operation === "cropMove" || event.operation === "cropResize";
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
      (event.operation === "radius" &&
        Math.abs(event.scale - active.lastScale) > 1e-9);
    const differsFromLastUpdate =
      Math.abs(event.deltaX - active.lastDeltaX) > 1e-9 ||
      Math.abs(event.deltaY - active.lastDeltaY) > 1e-9 ||
      event.edges !== active.lastEdges ||
      ((event.operation === "resize" ||
        event.operation === "radius" ||
        event.operation === "cropResize") &&
        Math.abs(event.scale - active.lastScale) > 1e-9);
    const cropOperation =
      event.operation === "cropMove" || event.operation === "cropResize";
    const shouldApply =
      event.phase === "end" ? cropOperation || differsFromLastUpdate : changed;
    // Native gesture deltas arrive as a share of the canvas.
    const moveX = event.deltaX * output.width;
    const moveY = event.deltaY * output.height;
    const cropX = active.snapshot.cropX + moveX;
    const cropY = active.snapshot.cropY + moveY;
    let next: ScreenshotOutputSettings;
    if (event.operation === "cropMove" || event.operation === "cropResize") {
      const source = items.find((item) => item.id === active.itemId);
      if (!source) return;
      next = applyScreenshotCropGesture({
        ...event,
        operation: event.operation,
        output,
        settings: active.snapshot,
        source,
      });
      if (event.phase === "end")
        next = commitScreenshotCrop(active.snapshot, next, source);
    } else if (event.operation === "radius") {
      next = {
        ...active.snapshot,
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
        ...active.snapshot,
        cropHeight: active.snapshot.cropHeight * scale,
        cropWidth: active.snapshot.cropWidth * scale,
        cropX,
        cropY,
        imageWidth: active.snapshot.imageWidth * scale,
        imageX: transform(active.snapshot.imageX, active.snapshot.cropX, cropX),
        imageY: transform(active.snapshot.imageY, active.snapshot.cropY, cropY),
      };
    } else {
      next = {
        ...active.snapshot,
        cropX,
        cropY,
        imageX: active.snapshot.imageX + moveX,
        imageY: active.snapshot.imageY + moveY,
      };
    }
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
      if (cropOperation) onCropChangeEnd?.(next.sourceCrop);
      selectionGestureRef.current = null;
      requestAnimationFrame(editGesture.endGesture);
      if (event.operation === "radius") onRadiusChangeEnd?.();
    }
    return;
  };
  // A finished arrow gesture is one edit: the native tool drew every frame of
  // it, and only the list it ends with reaches the document.
  const annotationChange = (event: ScreenshotAnnotationChangeEvent) => {
    onSelectedAnnotationChange?.(event.selectedAnnotationId);
    const itemOutput = workspaceOutput?.items[event.paneIndex];
    if (!workspaceOutput || !itemOutput) return;
    const held = screenshotWorkspaceItemOutput(workspaceOutput, itemOutput.id);
    const annotations = event.annotations;
    // Choosing an arrow reports the same list it was already holding. That is
    // a selection, not an edit, and must not land in the undo history.
    if (JSON.stringify(held.annotations) !== JSON.stringify(annotations))
      onOutputChange?.({ ...held, annotations }, itemOutput.id);
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
            cropMode: isEditing,
            paneIndex: selectedItemIndex,
            radiusPercent: selectedItemOutput.radiusPercent,
            ...normalizedScreenshotSelection(
              screenshotLayout(selectedItem, selectedItemOutput),
              output,
              isEditing ? "crop" : "select",
            ),
          }
        : null;
  const selectionTargets =
    (isSelecting || isEditing) && workspaceOutput
      ? screenshotSelectionTargets(items, workspaceOutput, {
          ...output,
          cropMode: isEditing,
        })
      : null;
  const { fitDuringResize, fitPreview, setFitBasis } =
    useScreenshotPreviewSurface({
      annotationCounterAngle,
      annotationDefaults,
      annotationTool,
      artifactId,
      canvasRef: nativeFrameRef,
      interactionOutput: workspaceOutput,
      isEditorSuspended: isSaving || isExportOpen,
      isEnabled: workspaceOutput !== undefined,
      onAnnotationChange: annotationChange,
      onAnnotationHover,
      onPaneFitChange,
      onSelectionChange: (paneIndex) => {
        if (paneIndex === null) return;
        const itemOutput = workspaceOutput?.items[paneIndex];
        if (itemOutput) onItemSelect?.(itemOutput.id);
      },
      onSelectionGesture: selectionGesture,
      onZoomChange,
      output: previewOutput,
      paneCount: orderedItems.length,
      selectedAnnotationId,
      selection: selectionOverlay,
      selectionTargets,
      sourceKey: orderedItems
        .map((item) => item.id)
        .sort((first, second) => first - second)
        .join(":"),
      zoomRequest,
    });
  useRegisterPreviewFit({ fitDuringResize, fitPreview, setFitBasis });
  return (
    <div
      aria-label={alt}
      className={`relative flex min-h-0 grow overflow-hidden ${isDrawingArrows ? "cursor-crosshair" : isSelecting ? "cursor-move" : "cursor-grab"}`}
      data-recording-preview-viewport
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
