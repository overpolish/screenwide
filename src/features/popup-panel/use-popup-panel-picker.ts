// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef } from "react";

import { hidePopupPanel, panelStartX, showPopupPanel } from "./api";
import {
  activePopupPanel,
  PopupPanelContent,
  SHARED_POPUP_PANEL,
  usePopupPanelStore,
} from "./store";
import { boundsAnchor } from "./use-popup-menu";

/** The gap between the trigger and the panel under it, in logical px. */
const ANCHOR_GAP = 4;

type PopupPanelPickerOptions = {
  /** What the panel shows while open; a change reaches the open panel. */
  content: PopupPanelContent;
  /** Names the panel, and the picks it reports. */
  id: string;
  label: string;
  /** A pick reported from the panel. */
  onPick: (value: string) => void;
  /** A picker holding one choice closes once it is made; one tried again
   * and again, as a colour is, stays open until put away. */
  closesOnPick?: boolean;
};

/** Reports `value` picked in the panel opened under `id`, from the panel
 * window, to whichever window opened it. */
export const reportPopupPanelPick = (id: string, value: string) => {
  usePopupPanelStore.getState().select({
    id,
    panel: SHARED_POPUP_PANEL,
    pressedId: value,
    selectedIds: [value],
  });
};

const putAway = async () => {
  const state = usePopupPanelStore.getState();
  const active = activePopupPanel(state, SHARED_POPUP_PANEL);
  state.close(SHARED_POPUP_PANEL);
  await hidePopupPanel(active?.focusContents ?? false, SHARED_POPUP_PANEL);
};

/**
 * Something to choose from, drawn in the shared panel window under its
 * trigger, rather than in the window the trigger is in: a tool panel is
 * sized to its own controls, so anything hung over them would be cut off at
 * its edge. The panel is fitted to what it shows.
 */
export function usePopupPanelPicker({
  closesOnPick = false,
  content,
  id,
  label,
  onPick,
}: PopupPanelPickerOptions) {
  const isOpen = usePopupPanelStore(
    (state) => activePopupPanel(state, SHARED_POPUP_PANEL)?.id === id,
  );
  const lastSelection = usePopupPanelStore((state) => state.lastSelection);
  const handledEventRef = useRef(lastSelection?.eventId ?? null);
  const pickRef = useRef(onPick);
  useEffect(() => {
    pickRef.current = onPick;
  });

  useEffect(() => {
    if (
      !lastSelection ||
      lastSelection.id !== id ||
      lastSelection.eventId === handledEventRef.current
    )
      return;
    handledEventRef.current = lastSelection.eventId;
    if (lastSelection.selectedIds.length > 0)
      pickRef.current(lastSelection.selectedIds[0]);
    if (closesOnPick) void putAway();
  }, [closesOnPick, id, lastSelection]);

  // What the panel shows follows what it picks, while it is open.
  const contentKey = JSON.stringify(content);
  useEffect(() => {
    const state = usePopupPanelStore.getState();
    const active = activePopupPanel(state, SHARED_POPUP_PANEL);
    if (active?.id !== id || JSON.stringify(active.content) === contentKey)
      return;
    state.open(SHARED_POPUP_PANEL, {
      ...active,
      content: JSON.parse(contentKey) as PopupPanelContent,
    });
  }, [contentKey, id]);

  /** Opens the panel `size` under `trigger`, or puts it away if open. */
  const toggle = async (
    trigger: HTMLElement,
    size: { height: number; width: number },
    focusContents: boolean,
  ) => {
    // The panel is a window of its own, which a story has none of.
    if (!isTauri()) return;
    if (isOpen) {
      await putAway();
      return;
    }
    const bounds = trigger.getBoundingClientRect();
    usePopupPanelStore
      .getState()
      .open(SHARED_POPUP_PANEL, { content, focusContents, id, label });
    await showPopupPanel({
      anchor: boundsAnchor(bounds),
      fitted: true,
      focusContents,
      offset: new LogicalPosition(
        panelStartX({ width: bounds.width, x: bounds.left }, size.width),
        bounds.bottom + ANCHOR_GAP,
      ),
      parentWindowLabel: getCurrentWindow().label,
      size: new LogicalSize(size.width, size.height),
      triggerId: id,
    });
  };

  return { isOpen, toggle };
}
