// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

import { t } from "../../../i18n/i18n";
import {
  type PopupMenuAnchor,
  pointerAnchor,
  usePopupMenu,
} from "../../popup-panel/use-popup-menu";
import {
  addCueMenuItem,
  addEditorCueFromMenu,
} from "../../teleprompter/cues/editor/use-editor-cue";
import {
  annotationArrangeItems,
  annotationArrangementPicked,
} from "../annotations/annotation-arrange-items";
import {
  Arrangement,
  availableArrangements,
} from "../annotations/annotation-order";
import {
  formatSyncMenu,
  pickedFormatSync,
} from "../recording/formats/format-item-menu";
import { useFrameFormats } from "../recording/formats/frame-formats-context";

import { layerSyncId } from "./formats/screenshot-show-in";
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
 * then adding the screenshot to a script as a cue, then Delete; the native
 * side has already selected the layer the press landed on. Every picture can
 * be deleted, the last leaving a blank canvas; the blank layer itself never
 * is.
 */
export function useScreenshotLayerActions({
  artifactId,
  isBlank,
  onCanvasResize,
  onSelectedItemChange,
  screenshotOutput,
  selectedItemId,
}: {
  /** The screenshot open, which a cue adds whole. */
  artifactId: number;
  /** Whether a layer is a blank canvas's own (`screenshot-blank.ts`). */
  isBlank: (itemId: number) => boolean;
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
    if (!screenshotOutput || itemId === null || isBlank(itemId)) return;
    const result = deleteScreenshotLayer({
      itemId,
      settings: screenshotOutput,
    });
    if (!result) return;
    onCanvasResize?.(result.settings);
    onSelectedItemChange?.(result.nextSelectedItemId);
  };
  // Where the menu opened, for the script picker a cue may open there.
  const anchorRef = useRef<PopupMenuAnchor | null>(null);
  const cueItem = addCueMenuItem();
  const formats = useFrameFormats();
  const openMenu = usePopupMenu({
    idPrefix: "screenshot-layer:",
    label: t("editor-layer-actions"),
    mode: "menu",
    onSelect: (itemId, layerId) => {
      if (itemId === cueItem.id) {
        if (anchorRef.current)
          addEditorCueFromMenu("screenshot", artifactId, anchorRef.current);
        return;
      }
      const id = Number(layerId);
      if (!Number.isInteger(id)) return;
      if (pickedFormatSync(formats, itemId, layerSyncId(id))) return;
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
    const anchor = pointerAnchor(x, y);
    anchorRef.current = anchor;
    // A blank canvas's own layer holds what is drawn on it, which each
    // annotation's own menu sets apart instead.
    const formatMenu: ReturnType<typeof formatSyncMenu> = isBlank(layer.id)
      ? { items: [], selectedIds: [] }
      : formatSyncMenu(formats, layerSyncId(layer.id));
    void openMenu({
      anchor,
      context: String(layer.id),
      items: [
        ...annotationArrangeItems(
          availableArrangements(items, paneIndex, meet),
        ),
        cueItem,
        ...(isBlank(layer.id)
          ? []
          : [
              {
                id: DELETE_ITEM,
                label: t("editor-delete"),
                shortcut: "Backspace",
              },
            ]),
        ...formatMenu.items,
      ],
      selectedIds: formatMenu.selectedIds,
      ...(formatMenu.scrollingSection === undefined
        ? {}
        : { scrollingSection: formatMenu.scrollingSection }),
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
