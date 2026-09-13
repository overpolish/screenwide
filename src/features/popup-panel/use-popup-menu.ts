// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef } from "react";

import { hidePopupPanel, showPopupPanel } from "./api";
import { initialPopupPanelHeight } from "./layout";
import {
  activePopupPanel,
  PopupPanelItem,
  SHARED_POPUP_PANEL,
  usePopupPanelStore,
} from "./store";

/** What the menu hangs off, in logical px relative to the parent window's
 * content. Rust excludes a press inside it from outside dismissal. */
export type PopupMenuAnchor = {
  height: number;
  width: number;
  x: number;
  y: number;
};

/** A right click has no trigger to hang off, so the point itself is the
 * anchor and the menu opens just under the pointer. */
export const pointerAnchor = (x: number, y: number): PopupMenuAnchor => ({
  height: 1,
  width: 1,
  x,
  y,
});

/** An element's own bounds, for a menu opened on the thing it acts upon. */
export const boundsAnchor = (bounds: DOMRect): PopupMenuAnchor => ({
  height: bounds.height,
  width: bounds.width,
  x: bounds.left,
  y: bounds.top,
});

/** The gap between the anchor and the menu under it, in logical px. */
const ANCHOR_GAP = 4;

type PopupMenuOptions = {
  /** Heads every id this menu opens under, so a selection reaching the shared
   * panel is recognized as this menu's rather than another pop-up's. What
   * follows the prefix names what the menu was opened on. */
  idPrefix: string;
  label: string;
  /** `select` ticks the current choice and keeps it; `menu` runs an action. */
  mode: "menu" | "select";
  /** `context` is whatever `open` was given, carried back through the id. */
  onSelect: (itemId: string, context: string) => void;
  width: number;
};

type PopupMenuRequest = {
  anchor: PopupMenuAnchor;
  items: PopupPanelItem[];
  /** What the menu was opened on, handed back to `onSelect`. */
  context?: string;
  selectedIds?: string[];
};

/**
 * Puts away the menu with this prefix if it is the one showing: a menu opened
 * on something that has since gone away, such as a layer whose tool was put
 * down, must not outlive what it was opened on.
 */
export const dismissPopupMenu = async (idPrefix: string) => {
  const state = usePopupPanelStore.getState();
  const active = activePopupPanel(state, SHARED_POPUP_PANEL);
  if (!active?.id.startsWith(idPrefix)) return;
  state.close(SHARED_POPUP_PANEL);
  await hidePopupPanel(active.focusContents, SHARED_POPUP_PANEL);
};

/**
 * A context menu drawn in the app's own panel window.
 *
 * The menu opens in the shared listbox with the current window as its parent,
 * so it floats over whatever it was opened on and is clamped to the monitor
 * rather than to the window. Picking an item dismisses the panel from its own
 * side; the parent drops its copy here so the next right click opens again.
 */
export function usePopupMenu({
  idPrefix,
  label,
  mode,
  onSelect,
  width,
}: PopupMenuOptions) {
  const lastSelection = usePopupPanelStore((state) => state.lastSelection);
  const close = usePopupPanelStore((state) => state.close);
  const open = usePopupPanelStore((state) => state.open);
  const handledEventRef = useRef(lastSelection?.eventId ?? null);
  const selectRef = useRef(onSelect);

  useEffect(() => {
    selectRef.current = onSelect;
  });

  useEffect(() => {
    if (
      !lastSelection ||
      !lastSelection.id.startsWith(idPrefix) ||
      lastSelection.eventId === handledEventRef.current
    ) {
      return;
    }
    handledEventRef.current = lastSelection.eventId;
    const state = usePopupPanelStore.getState();
    const active = activePopupPanel(state, SHARED_POPUP_PANEL);
    if (active?.id === lastSelection.id) {
      state.close(SHARED_POPUP_PANEL);
      void hidePopupPanel(active.focusContents, SHARED_POPUP_PANEL);
    }
    // A toggle reports the whole tick list; these menus hold one choice, so
    // an empty list is a dismissal rather than a pick.
    if (lastSelection.selectedIds.length > 0)
      selectRef.current(
        lastSelection.selectedIds[0],
        lastSelection.id.slice(idPrefix.length),
      );
  }, [idPrefix, lastSelection]);

  return async ({
    anchor,
    context = "",
    items,
    selectedIds = [],
  }: PopupMenuRequest) => {
    // Nothing applies here, so there is no menu to show rather than an empty
    // one saying so.
    if (items.length === 0) return;
    // The menu is a window of its own, so a story right-clicking a real
    // component outside the app has nothing to open and says so by doing
    // nothing.
    if (!isTauri()) return;

    const id = `${idPrefix}${context}`;
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
        mode,
        selectedIds,
        selectionMode: "single",
      },
      focusContents: false,
      id,
      label,
    });
    await showPopupPanel({
      anchor,
      focusContents: false,
      offset: new LogicalPosition(
        anchor.x,
        anchor.y + anchor.height + ANCHOR_GAP,
      ),
      parentWindowLabel: getCurrentWindow().label,
      size: new LogicalSize(width, initialPopupPanelHeight(items.length)),
      triggerId: id,
    });
  };
}
