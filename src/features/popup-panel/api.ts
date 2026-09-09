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
};

export const showPopupPanel = ({
  anchor,
  focusContents,
  offset,
  parentWindowLabel,
  size,
  triggerId,
}: ShowPopupPanelOptions) =>
  invoke<null>("show_standalone_listbox", {
    anchor: anchor ?? null,
    focusContents,
    offset,
    parentWindowLabel,
    size,
    triggerId,
  });

export const hidePopupPanel = (returnFocus = false) =>
  invoke<null>("hide_standalone_listbox", { returnFocus });
