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

/**
 * Where a panel `width` wide starts so it hangs from the anchor's starting
 * edge, as a native menu does: the anchor's left in a left-to-right window,
 * its right in a right-to-left one.
 */
export const panelStartX = (
  anchor: { width: number; x: number },
  width: number,
) =>
  document.documentElement.dir === "rtl"
    ? anchor.x + anchor.width - width
    : anchor.x;

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
    // The page's own height says where it begins in its window.
    pageHeight: window.innerHeight,
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
 * editor resize moves the corner it sits in; the editor's island covers the
 * preview, so it takes the preview's new `size` too. A fresh
 * `showPopupPanel` would re-attach and re-order the panel window for what is
 * a move.
 */
export const movePopupPanel = ({
  offset,
  panel,
  parentWindowLabel,
  size,
}: {
  offset: LogicalPosition;
  parentWindowLabel: string;
  panel?: string;
  size?: LogicalSize;
}) =>
  invoke<null>("move_standalone_listbox", {
    offset,
    pageHeight: window.innerHeight,
    panel: panel ?? null,
    parentWindowLabel,
    size: size ?? null,
  });

/** A part of a panel, in CSS pixels from its window's top left. */
export type PopupPanelPart = {
  height: number;
  width: number;
  x: number;
  y: number;
};

/** Where a panel larger than what it shows has its parts, so clicks on them
 * reach it and clicks anywhere else in its window pass through; null makes
 * it take every click again. */
export const setPopupPanelHitArea = (
  panel: string,
  parts: PopupPanelPart[] | null,
) => invoke<null>("set_standalone_listbox_hit_area", { panel, rects: parts });

/** The panel windows showing something right now, by label. */
export const openPopupPanels = () =>
  invoke<string[]>("open_standalone_listboxes");

export const hidePopupPanel = (returnFocus = false, panel?: string) =>
  invoke<null>("hide_standalone_listbox", {
    panel: panel ?? null,
    returnFocus,
  });
