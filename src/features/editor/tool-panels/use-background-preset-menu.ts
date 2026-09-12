// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef } from "react";

import { hidePopupPanel, showPopupPanel } from "../../popup-panel/api";
import { initialPopupPanelHeight } from "../../popup-panel/layout";
import {
  activePopupPanel,
  PopupPanelItem,
  SHARED_POPUP_PANEL,
  usePopupPanelStore,
} from "../../popup-panel/store";

/** One menu per saved background, so a selection names the tile it came
 * from without the panel having to carry it back. */
const MENU_PREFIX = "background-preset:";
/** The one action reads wider than the swatch it hangs off. */
const MENU_WIDTH = 180;

const items: PopupPanelItem[] = [
  { icon: "trash", id: "remove", label: "Remove Preset" },
];

/**
 * A right click on a saved background, answered with the app's own menu.
 *
 * The menu opens in the shared listbox window with the tool panel as its
 * parent, so it floats over the panel rather than inside it. An outside press
 * dismisses the menu alone: a tool panel is sticky, and stays where it is.
 */
export function useBackgroundPresetMenu(onRemove: (presetId: string) => void) {
  const lastSelection = usePopupPanelStore((state) => state.lastSelection);
  const close = usePopupPanelStore((state) => state.close);
  const open = usePopupPanelStore((state) => state.open);
  const handledEventRef = useRef(lastSelection?.eventId ?? null);
  const removeRef = useRef(onRemove);

  useEffect(() => {
    removeRef.current = onRemove;
  });

  useEffect(() => {
    if (
      !lastSelection ||
      !lastSelection.id.startsWith(MENU_PREFIX) ||
      lastSelection.eventId === handledEventRef.current
    ) {
      return;
    }
    handledEventRef.current = lastSelection.eventId;
    const state = usePopupPanelStore.getState();
    const active = activePopupPanel(state, SHARED_POPUP_PANEL);
    // The panel is a menu, not a select: picking an action dismisses it.
    if (active?.id === lastSelection.id) {
      state.close(SHARED_POPUP_PANEL);
      void hidePopupPanel(active.focusContents, SHARED_POPUP_PANEL);
    }
    if (lastSelection.selectedIds[0] === "remove") {
      removeRef.current(lastSelection.id.slice(MENU_PREFIX.length));
    }
  }, [lastSelection]);

  return async (presetId: string, anchor: DOMRect) => {
    const id = `${MENU_PREFIX}${presetId}`;
    const current = activePopupPanel(
      usePopupPanelStore.getState(),
      SHARED_POPUP_PANEL,
    );
    if (current?.id === id) {
      close(SHARED_POPUP_PANEL);
      await hidePopupPanel(current.focusContents, SHARED_POPUP_PANEL);
      return;
    }
    open(SHARED_POPUP_PANEL, {
      content: {
        items,
        kind: "list",
        mode: "menu",
        selectedIds: [],
        selectionMode: "single",
      },
      focusContents: false,
      id,
      label: "Background actions",
    });
    await showPopupPanel({
      anchor: {
        height: anchor.height,
        width: anchor.width,
        x: anchor.left,
        y: anchor.top,
      },
      focusContents: false,
      offset: new LogicalPosition(anchor.left, anchor.bottom + 4),
      parentWindowLabel: getCurrentWindow().label,
      size: new LogicalSize(MENU_WIDTH, initialPopupPanelHeight(items.length)),
      triggerId: id,
    });
  };
}
