// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AnnotationChoice,
  AnnotationClipboard,
  copyAnnotations,
  movedAnnotations,
  pastedAnnotations,
} from "../annotations/annotation-clipboard";
import { CaptureSource } from "../layer-measure";

import {
  ScreenshotOutputSettings,
  screenshotWorkspaceItemOutput,
  ScreenshotWorkspaceOutputSettings,
} from "./screenshot-output";

/** How far an arrow key nudges the chosen annotations, in points, and how
 * far with Shift. */
const NUDGE_POINTS = { coarse: 10, fine: 1 } as const;

/**
 * Copy, cut, paste, duplicate, choose all and nudge for the screenshot's
 * annotations: on the layer in hand, or with none in hand the layer a tool
 * takes up (`targetItemId`). Each edit is one commit of that layer, so one
 * undo step, and what it adds is chosen, so it can be moved at once.
 */
export function screenshotAnnotationClipboard({
  annotations,
  items,
  onOutputChange,
  onSelectedItemChange,
  selectedItemId,
  targetItemId,
  workspace,
}: {
  annotations: AnnotationChoice;
  items: readonly (CaptureSource & { id: number })[];
  selectedItemId: number | null;
  /** The layer a paste lands on with no layer in hand. */
  targetItemId: number | null;
  workspace: ScreenshotWorkspaceOutputSettings | undefined;
  onOutputChange?: (settings: ScreenshotOutputSettings, itemId: number) => void;
  onSelectedItemChange?: (itemId: number | null) => void;
}) {
  const layerOf = (itemId: number | null) => {
    const item = items.find((candidate) => candidate.id === itemId);
    if (!item || !workspace) return null;
    return { item, output: screenshotWorkspaceItemOutput(workspace, item.id) };
  };
  const selectedLayer = layerOf(selectedItemId);
  const chosen = selectedLayer
    ? selectedLayer.output.annotations.filter(({ id }) =>
        annotations.selectedIds.has(id),
      )
    : [];
  const clipboardOfChosen = (): AnnotationClipboard | null =>
    selectedLayer && chosen.length > 0
      ? {
          capture: {
            height: selectedLayer.item.height,
            scaleFactor: selectedLayer.item.scaleFactor,
            width: selectedLayer.item.width,
          },
          items: chosen.map((annotation) => ({ annotation })),
        }
      : null;
  const copy = () => {
    const clipboard = clipboardOfChosen();
    if (clipboard)
      copyAnnotations(clipboard).catch((cause: unknown) => {
        console.error("Could not copy the annotations", cause);
      });
    return clipboard;
  };
  const paste = (clipboard: AnnotationClipboard) => {
    const layer = layerOf(selectedItemId ?? targetItemId);
    if (!layer) return;
    const pasted = pastedAnnotations(
      clipboard,
      layer.item,
      layer.output.annotations,
    );
    onOutputChange?.(
      {
        ...layer.output,
        annotations: [...layer.output.annotations, ...pasted],
      },
      layer.item.id,
    );
    if (layer.item.id !== selectedItemId) onSelectedItemChange?.(layer.item.id);
    annotations.onSelectedIdsChange(pasted.map(({ id }) => id));
  };
  const hasChosen = chosen.length > 0;
  return {
    /** ⌘C with annotations chosen; unset with none, where the screenshot
     * itself is copied. */
    copy: hasChosen
      ? () => {
          copy();
        }
      : undefined,
    cut: hasChosen
      ? () => {
          if (copy()) annotations.deleteSelected();
        }
      : undefined,
    duplicate: hasChosen
      ? () => {
          const clipboard = clipboardOfChosen();
          if (clipboard) paste(clipboard);
        }
      : undefined,
    hasChosen,
    nudge: (directionX: number, directionY: number, coarse: boolean) => {
      if (!selectedLayer || !hasChosen || selectedItemId === null) return;
      const pixels =
        (coarse ? NUDGE_POINTS.coarse : NUDGE_POINTS.fine) *
        selectedLayer.item.scaleFactor;
      const moved = new Map(
        movedAnnotations(chosen, {
          x: directionX * pixels,
          y: directionY * pixels,
        }).map((annotation) => [annotation.id, annotation]),
      );
      onOutputChange?.(
        {
          ...selectedLayer.output,
          annotations: selectedLayer.output.annotations.map(
            (annotation) => moved.get(annotation.id) ?? annotation,
          ),
        },
        selectedItemId,
      );
    },
    paste,
    /** ⌘A: every annotation on the layer in hand, or on the layer a tool
     * takes up, which is then in hand. */
    selectAll: () => {
      const layer = layerOf(selectedItemId ?? targetItemId);
      if (!layer || layer.output.annotations.length === 0) return;
      if (layer.item.id !== selectedItemId)
        onSelectedItemChange?.(layer.item.id);
      annotations.onSelectedIdsChange(
        layer.output.annotations.map(({ id }) => id),
      );
    },
  };
}
