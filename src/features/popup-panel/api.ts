// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";

/** The trigger's bounds in logical px, relative to the parent window's
 * content, the way `offset` is expressed. Rust excludes a press inside it
 * from outside dismissal so the trigger can toggle the panel on mouse-up. */
type PopupPanelAnchor = {
  height: number;
  width: number;
  x: number;
  y: number;
};

type ShowPopupPanelOptions = {
  focusContents: boolean;
  offset: LogicalPosition;
  parentWindowLabel: string;
  size: LogicalSize;
  triggerId: string;
  anchor?: PopupPanelAnchor;
  /** Which panel window to show this in. Omitted means the shared listbox
   * every pop-up button borrows; an editor names its own tool panel. */
  panel?: string;
  /** A panel the user works alongside, which an outside press must not take
   * away. It closes on Escape, on its own trigger, or with its parent
   * window. */
  sticky?: boolean;
};

export const showPopupPanel = ({
  anchor,
  focusContents,
  offset,
  panel,
  parentWindowLabel,
  size,
  sticky = false,
  triggerId,
}: ShowPopupPanelOptions) =>
  invoke<null>("show_standalone_listbox", {
    anchor: anchor ?? null,
    focusContents,
    offset,
    panel: panel ?? null,
    parentWindowLabel,
    size,
    sticky,
    triggerId,
  });

/**
 * Re-places an open panel against its parent's content, without re-showing it.
 *
 * A tool panel hangs off the preview rather than the window frame, so an
 * editor resize moves the corner it sits in. Only its position changes: a
 * fresh `showPopupPanel` would re-attach and re-order the panel window for
 * what is a move.
 */
export const movePopupPanel = (
  parentWindowLabel: string,
  offset: LogicalPosition,
  panel?: string,
) =>
  invoke<null>("move_standalone_listbox", {
    offset,
    panel: panel ?? null,
    parentWindowLabel,
  });

export const hidePopupPanel = (returnFocus = false, panel?: string) =>
  invoke<null>("hide_standalone_listbox", {
    panel: panel ?? null,
    returnFocus,
  });
