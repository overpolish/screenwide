// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

/** Sent when the tooltip window, already loaded, is asked to say something
 * else. */
const TOOLTIP_TEXT_EVENT = "tooltip://text";

/** What a tooltip says. The shortcut travels apart from the label so the
 * window can draw it as keycaps rather than as run-on text. It is written in
 * the hotkey format Settings stores (`CommandOrControl+KeyZ`, `Shift+KeyZ`,
 * `Backspace`), so each platform names its own modifiers. */
export type TooltipContent = {
  label: string;
  shortcut?: string;
};

/** The control a tooltip describes, in the asking window's own logical
 * pixels. */
export type TooltipAnchor = {
  height: number;
  width: number;
  x: number;
  y: number;
};

/** Crossing from one control to its neighbour closes the first tooltip a
 * moment before the second opens. Hiding on that first close would blink the
 * window out and back in, so the hide waits long enough for the next tooltip
 * to claim it. */
const HANDOVER_MS = 80;
let pendingHide: ReturnType<typeof setTimeout> | undefined;

const cancelPendingHide = () => {
  if (pendingHide === undefined) return;
  clearTimeout(pendingHide);
  pendingHide = undefined;
};

/** Asks for a tooltip over `anchor`. The window it belongs to is named so the
 * anchor can be read as a place on the screen. */
export const showTooltip = async (
  content: TooltipContent,
  anchor: TooltipAnchor,
) => {
  cancelPendingHide();
  await invoke<null>("show_tooltip", {
    anchor,
    content,
    // The page's own height says where it begins in its window, which a
    // window with a title bar of its own does not report.
    pageHeight: window.innerHeight,
    parentWindowLabel: getCurrentWindow().label,
  });
};

/** Takes the tooltip away, unless another one claims it first. */
export const hideTooltip = () => {
  cancelPendingHide();
  pendingHide = setTimeout(() => {
    pendingHide = undefined;
    invoke<null>("hide_tooltip").catch((cause: unknown) => {
      console.error("Could not hide the tooltip", cause);
    });
  }, HANDOVER_MS);
};

/** Reports the size the tooltip's words need; the window is sized to it,
 * placed under its anchor and shown. */
export const fitTooltip = async (width: number, height: number) => {
  await invoke<null>("fit_tooltip", { height, width });
};

/** What the tooltip should say right now, read once on mount. */
export const getTooltip = async () =>
  invoke<TooltipContent | null>("get_tooltip");

/** The window stays loaded between tooltips, so later ones arrive here. */
export const listenToTooltipText = async (
  onText: (content: TooltipContent) => void,
): Promise<UnlistenFn> =>
  listen<TooltipContent>(TOOLTIP_TEXT_EVENT, ({ payload }) => {
    onText(payload);
  });
