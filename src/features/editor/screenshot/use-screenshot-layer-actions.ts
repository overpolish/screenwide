// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

import { pointerAnchor, usePopupMenu } from "../../popup-panel/use-popup-menu";
import {
  annotationArrangeItems,
  annotationArrangementPicked,
} from "../annotations/annotation-arrange-items";
import {
  Arrangement,
  availableArrangements,
} from "../annotations/annotation-order";

import {
  deleteScreenshotLayer,
  moveScreenshotLayer,
} from "./screenshot-layer-actions";
import { ScreenshotWorkspaceOutputSettings } from "./screenshot-output";

const MENU_WIDTH = 200;
const DELETE_ITEM = "layer:delete";

// Every layer is drawn with every other, so any two meet.
const meet = () => true;

/**
 * The workspace's layers moved through the stacking or taken away, by the
 * keyboard or from the menu a right press on a bare layer opens. The menu
 * offers the same moves, under the same words and keys, as an annotation's,
 * then Delete while another layer would be left; the native side has
 * already selected the layer the press landed on.
 */
export function useScreenshotLayerActions({
  onCanvasResize,
  onSelectedItemChange,
  screenshotOutput,
  selectedItemId,
}: {
  screenshotOutput: ScreenshotWorkspaceOutputSettings | undefined;
  selectedItemId: number | null;
  onCanvasResize?: (settings: ScreenshotWorkspaceOutputSettings) => void;
  onSelectedItemChange?: (itemId: number | null) => void;
}) {
  const moveLayer = (arrangement: Arrangement, itemId = selectedItemId) => {
    if (!screenshotOutput || itemId === null) return;
    const next = moveScreenshotLayer({
      arrangement,
      itemId,
      settings: screenshotOutput,
    });
    if (next !== screenshotOutput) onCanvasResize?.(next);
  };
  const deleteLayer = (itemId = selectedItemId) => {
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
  const openMenu = usePopupMenu({
    idPrefix: "screenshot-layer:",
    label: "Layer actions",
    mode: "menu",
    onSelect: (itemId, layerId) => {
      const id = Number(layerId);
      if (!Number.isInteger(id)) return;
      if (itemId === DELETE_ITEM) {
        deleteLayer(id);
        return;
      }
      const arrangement = annotationArrangementPicked(itemId);
      if (arrangement) moveLayer(arrangement, id);
    },
    width: MENU_WIDTH,
  });
  const openRef = useRef(
    (_: { paneIndex: number; x: number; y: number }) => {},
  );
  openRef.current = ({ paneIndex, x, y }) => {
    const items = screenshotOutput?.items ?? [];
    if (paneIndex < 0 || paneIndex >= items.length) return;
    const layer = items[paneIndex];
    const moves = [
      ...annotationArrangeItems(availableArrangements(items, paneIndex, meet)),
      ...(items.length > 1
        ? [{ id: DELETE_ITEM, label: "Delete", shortcut: "Backspace" }]
        : []),
    ];
    if (moves.length === 0) return;
    void openMenu({
      anchor: pointerAnchor(x, y),
      context: String(layer.id),
      items: moves,
    });
  };
  useEffect(() => {
    // The subscription lands after a hop; a cleanup that runs first must
    // still let go of it.
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<unknown>("screenshot-preview://layer-menu", (event) => {
      const payload = event.payload;
      if (disposed || typeof payload !== "object" || payload === null) return;
      const { paneIndex, x, y } = payload as Record<string, unknown>;
      if (
        typeof paneIndex === "number" &&
        Number.isInteger(paneIndex) &&
        typeof x === "number" &&
        typeof y === "number" &&
        Number.isFinite(x) &&
        Number.isFinite(y)
      )
        openRef.current({ paneIndex, x, y });
    }).then((dispose) => {
      if (disposed) dispose();
      else unlisten = dispose;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
  return { deleteLayer, moveLayer };
}
