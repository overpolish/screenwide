// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";

import {
  applyScreenshotCropGesture,
  commitScreenshotCrop,
  uncroppedScreenshotPreviewOutput,
} from "../screenshot-crop";
import {
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
  fitScreenshotWorkspaceToItems,
  resizeScreenshotWorkspaceCanvasEdges,
  screenshotLayout,
  screenshotWorkspaceItemOutput,
  screenshotOutputDimensions,
} from "../screenshot-output";
import { applyScreenshotRecenterGesture } from "../screenshot-recenter";
import { useExportEditGesture } from "../use-export-edit-history";
import {
  ScreenshotSelectionGestureEvent,
  useScreenshotPreviewSurface,
} from "../use-screenshot-preview-surface";

import { PreviewPaneFit } from "./preview-transform";
import { normalizedScreenshotSelection } from "./screenshot-selection";

type PreviewViewportProps = {
  alt: string;
  artifactId: number;
  items: { height: number; id: number; width: number }[];
  naturalHeight: number;
  naturalWidth: number;
  isEditing?: boolean;
  isRecentering?: boolean;
  isResizingCanvas?: boolean;
  /** A save suspends native input so its DOM Cancel button remains clickable. */
  isSaving?: boolean;
  isSelecting?: boolean;
  onBackgroundRadiusChange?: (radiusPercent: number) => void;
  onBackgroundRadiusChangeEnd?: () => void;
  onCanvasResize?: (settings: ScreenshotWorkspaceOutputSettings) => void;
  onItemSelect?: (itemId: number) => void;
  onOutputChange?: (
    settings: ScreenshotOutputSettings,
    itemId?: number,
  ) => void;
  onPaneFitChange?: (fit: PreviewPaneFit) => void;
  onRadiusChangeEnd?: () => void;
  onRecenter?: () => void;
  onZoomChange?: (zoomPercent: number) => void;
  screenshotOutput?: ScreenshotWorkspaceOutputSettings;
  selectedItemId?: number | null;
  zoomPercent?: number;
};

const AUTO_FIT_MOVE_EDGE = 1 << 17;
const AUTO_FIT_COMMIT_EDGE = 1 << 18;

export function PreviewViewport({
  alt,
  artifactId,
  isEditing = false,
  isRecentering = false,
  isResizingCanvas = false,
  isSaving = false,
  isSelecting = false,
  items,
  naturalHeight,
  naturalWidth,
  onBackgroundRadiusChange,
  onBackgroundRadiusChangeEnd,
  onCanvasResize,
  onItemSelect,
  onOutputChange,
  onPaneFitChange,
  onRadiusChangeEnd,
  onRecenter,
  onZoomChange,
  screenshotOutput,
  selectedItemId = null,
  zoomPercent,
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
  const editGesture = useExportEditGesture();
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
          sources: items,
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
    if (event.operation === "recenterAction") return onRecenter?.();
    if (frameGesture(event)) return;
    if (event.phase === "begin") {
      const itemOutput = workspaceOutput?.items[event.paneIndex];
      const cropGesture =
        event.operation === "cropMove" || event.operation === "cropResize";
      if (
        (!isSelecting && !isRecentering && !(isEditing && cropGesture)) ||
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
    const cropX = active.snapshot.screenshotCropXPercent + event.deltaX * 100;
    const cropY = active.snapshot.screenshotCropYPercent + event.deltaY * 100;
    let next: ScreenshotOutputSettings;
    const recentered = isRecentering
      ? applyScreenshotRecenterGesture({
          deltaX: event.deltaX,
          deltaY: event.deltaY,
          edges: event.edges,
          operation: event.operation,
          scale: event.scale,
          settings: active.snapshot,
          source: items.find((item) => item.id === active.itemId),
        })
      : null;
    if (recentered) {
      next = recentered;
    } else if (
      event.operation === "cropMove" ||
      event.operation === "cropResize"
    ) {
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
        screenshotCropHeightPercent:
          active.snapshot.screenshotCropHeightPercent * scale,
        screenshotCropWidthPercent:
          active.snapshot.screenshotCropWidthPercent * scale,
        screenshotCropXPercent: cropX,
        screenshotCropYPercent: cropY,
        screenshotImageWidthPercent:
          active.snapshot.screenshotImageWidthPercent * scale,
        screenshotImageXPercent: transform(
          active.snapshot.screenshotImageXPercent,
          active.snapshot.screenshotCropXPercent,
          cropX,
        ),
        screenshotImageYPercent: transform(
          active.snapshot.screenshotImageYPercent,
          active.snapshot.screenshotCropYPercent,
          cropY,
        ),
      };
    } else {
      next = {
        ...active.snapshot,
        screenshotCropXPercent: cropX,
        screenshotCropYPercent: cropY,
        screenshotImageXPercent:
          active.snapshot.screenshotImageXPercent + event.deltaX * 100,
        screenshotImageYPercent:
          active.snapshot.screenshotImageYPercent + event.deltaY * 100,
      };
    }
    if (shouldApply) {
      const autoFit =
        !isRecentering &&
        event.operation === "move" &&
        (event.edges & AUTO_FIT_MOVE_EDGE) !== 0;
      if (autoFit) {
        const fitted = fitScreenshotWorkspaceToItems({
          initial: active.workspaceSnapshot,
          movedItemId: active.itemId,
          movedItemOutput: next,
          sources: orderedItems,
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
      selectionGestureRef.current = null;
      requestAnimationFrame(editGesture.endGesture);
      if (event.operation === "radius") onRadiusChangeEnd?.();
    }
    return;
  };
  const selectionOverlay =
    isResizingCanvas && workspaceOutput
      ? {
          layerId: 0xffffffff,
          paneIndex: 0,
          radiusPercent: workspaceOutput.backgroundRadiusPercent,
          rect: { height: 1, width: 1, x: 0, y: 0 },
        }
      : (isSelecting || isEditing || isRecentering) &&
          selectedItemIndex >= 0 &&
          selectedItem &&
          selectedItemOutput
        ? {
            cropMode: isEditing,
            paneIndex: selectedItemIndex,
            radiusPercent: selectedItemOutput.radiusPercent,
            recenterMode: isRecentering,
            ...normalizedScreenshotSelection(
              screenshotLayout(selectedItem, output, selectedItemOutput),
              output,
              isRecentering ? "recenter" : isEditing ? "crop" : "select",
            ),
          }
        : null;
  const selectionTargets =
    (isSelecting || isEditing || isRecentering) && workspaceOutput
      ? workspaceOutput.items.flatMap((itemOutput, paneIndex) => {
          const item = items.find(
            (candidate) => candidate.id === itemOutput.id,
          );
          if (!item) return [];
          const layout = screenshotLayout(item, output, itemOutput.output);
          return [
            {
              cropMode: isEditing,
              paneIndex,
              radiusPercent: itemOutput.output.radiusPercent,
              recenterMode: isRecentering,
              ...normalizedScreenshotSelection(
                layout,
                output,
                isRecentering ? "recenter" : isEditing ? "crop" : "select",
              ),
            },
          ];
        })
      : null;
  useScreenshotPreviewSurface({
    artifactId,
    canvasRef: nativeFrameRef,
    interactionOutput: workspaceOutput,
    isEditorSuspended: isSaving,
    isEnabled: workspaceOutput !== undefined,
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
    selection: selectionOverlay,
    selectionTargets,
    sourceKey: orderedItems
      .map((item) => item.id)
      .sort((first, second) => first - second)
      .join(":"),
    zoomPercent,
  });
  return (
    <div
      aria-label={alt}
      className={`relative flex min-h-0 grow overflow-hidden ${isSelecting || isRecentering ? "cursor-move" : "cursor-grab"}`}
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
