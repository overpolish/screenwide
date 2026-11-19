// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";

import {
  openPopupPanels,
  panelStartX,
  showPopupPanel,
} from "../../popup-panel/api";
import {
  type PopupPanelContent,
  SHARED_POPUP_PANEL,
  usePopupPanelStore,
} from "../../popup-panel/store";
import { type PopupMenuAnchor } from "../../popup-panel/use-popup-menu";

import { PICKER_SIZE } from "./picker/use-picker-size";

/** How often, and how many times, a menu's panel is asked whether it has
 * gone, in milliseconds and tries. */
const CLOSE_POLL_MS = 25;
const CLOSE_POLLS = 20;
/** Between what a panel opens from and the panel, in points. */
const ANCHOR_GAP = 4;

const pause = (ms: number) =>
  new Promise((resolve) => {
    window.setTimeout(resolve, ms);
  });

/**
 * Resolves once the menu a choice was made in has left the shared panel.
 * A cue's panel opens in that same panel, and the menu's going is told to
 * every window as it lands, which would put away a panel opened before it:
 * so the panel waits until the app has the menu closed, and a moment more
 * for that word to have reached this window.
 */
export async function afterMenuCloses() {
  for (let poll = 0; poll < CLOSE_POLLS; poll++) {
    if (!(await openPopupPanels()).includes(SHARED_POPUP_PANEL)) break;
    await pause(CLOSE_POLL_MS);
  }
  await pause(CLOSE_POLL_MS * 2);
}

/** Opens `content` in the shared panel at the picker's size, under
 * `anchor` in this window, named `label` and known by `id`. */
export async function openCuePanel({
  anchor,
  content,
  id,
  label,
}: {
  anchor: PopupMenuAnchor;
  content: PopupPanelContent;
  id: string;
  label: string;
}) {
  usePopupPanelStore.getState().open(SHARED_POPUP_PANEL, {
    content,
    focusContents: true,
    id,
    label,
  });
  await showPopupPanel({
    anchor,
    focusContents: true,
    offset: new LogicalPosition(
      panelStartX(anchor, PICKER_SIZE.width),
      anchor.y + anchor.height + ANCHOR_GAP,
    ),
    parentWindowLabel: getCurrentWindow().label,
    size: new LogicalSize(PICKER_SIZE.width, PICKER_SIZE.height),
    triggerId: id,
  });
}
