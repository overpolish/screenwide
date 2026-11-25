// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef } from "react";

import { hidePopupPanel, panelStartX, showPopupPanel } from "./api";
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

/** The gap between the anchor and the menu beside it, in logical px. */
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
  /** Hangs above the anchor rather than under it: a trigger at the foot of
   * a window, with the room above it. */
  opensAbove?: boolean;
  /** Opens at the full height of its items instead of scrolling past the
   * list ceiling. */
  showsAllItems?: boolean;
};

type PopupMenuRequest = {
  anchor: PopupMenuAnchor;
  items: PopupPanelItem[];
  /** What the menu was opened on, handed back to `onSelect`. */
  context?: string;
  /** The section, last in `items`, that scrolls on its own under the rest
   * (`PopupPanelListContent.scrollingSection`). */
  scrollingSection?: string;
  selectedIds?: string[];
};

/** Puts the menu opened under `idPrefix` away, if it is the one showing. */
export async function closePopupMenu(idPrefix: string) {
  const state = usePopupPanelStore.getState();
  const active = activePopupPanel(state, SHARED_POPUP_PANEL);
  if (!active?.id.startsWith(idPrefix)) return;
  state.close(SHARED_POPUP_PANEL);
  await hidePopupPanel(active.focusContents, SHARED_POPUP_PANEL);
}

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
  opensAbove = false,
  showsAllItems = false,
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
    const pressed =
      lastSelection.pressedId ??
      (lastSelection.selectedIds.length > 0
        ? lastSelection.selectedIds[0]
        : undefined);
    // A toggle flips its tick and leaves the menu open, so several can be
    // set in one visit; anything else is an action, and the menu goes.
    const isToggle =
      active?.content.kind === "list" &&
      active.content.items.some(
        (item) => item.id === pressed && item.togglesInPlace,
      );
    if (active?.id === lastSelection.id && !isToggle) {
      state.close(SHARED_POPUP_PANEL);
      void hidePopupPanel(active.focusContents, SHARED_POPUP_PANEL);
    }
    // An empty pick with nothing pressed is a dismissal.
    if (pressed !== undefined)
      selectRef.current(pressed, lastSelection.id.slice(idPrefix.length));
  }, [idPrefix, lastSelection]);

  return async ({
    anchor,
    context = "",
    items,
    scrollingSection,
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
    // A section only scrolls on its own under something that stays put.
    const scrolling = items.filter(
      (item) => item.section === scrollingSection,
    ).length;
    const scrolls =
      scrollingSection !== undefined &&
      scrolling > 0 &&
      scrolling < items.length;
    open(SHARED_POPUP_PANEL, {
      content: {
        items,
        kind: "list",
        mode,
        selectedIds,
        selectionMode: "single",
        showsAllItems,
        ...(scrolls ? { scrollingSection } : {}),
        ...(opensAbove ? { opensAbove } : {}),
      },
      focusContents: false,
      id,
      label,
    });
    // Opened at the height its rows and section headings take, so a headed
    // menu does not grow once it has measured itself.
    const height = initialPopupPanelHeight(
      items.length,
      new Set(items.flatMap((item) => item.section ?? [])).size,
      { scrollingCount: scrolls ? scrolling : 0, showsAllItems },
    );
    await showPopupPanel({
      anchor,
      focusContents: false,
      offset: new LogicalPosition(
        panelStartX(anchor, width),
        opensAbove
          ? anchor.y - ANCHOR_GAP - height
          : anchor.y + anchor.height + ANCHOR_GAP,
      ),
      parentWindowLabel: getCurrentWindow().label,
      size: new LogicalSize(width, height),
      triggerId: id,
    });
  };
}
