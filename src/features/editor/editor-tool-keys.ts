// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The plain letter keys that take up a tool or put its panel on screen.
 *
 * One table rather than a chain of conditions, so a tool added later states
 * its key in the same place as the rest and the hook that owns the window's
 * keyboard keeps the shape it has: the guards - a modifier, a repeat, a field
 * being typed in - are applied once, around the whole table.
 */
export type EditorToolKeys = {
  /** A: the arrow tool. */
  onArrowTool?: () => void;
  /** N: the counter tool. */
  onCounterTool?: () => void;
  onResizeCanvas?: () => void;
  onSelectTool?: () => void;
  onToggleBladeTool?: () => void;
  onToggleCrop?: () => void;
  /** M: the cursor panel, on or away. */
  onToggleCursorPanel?: () => void;
  /** K: the keyboard panel, on or away. */
  onToggleKeyboardPanel?: () => void;
  onTogglePlayback?: () => void;
  onToggleRangeTool?: () => void;
  /** S: timeline snapping, on or off. */
  onToggleSnap?: () => void;
};

/** What `code` does in this window, or null where it does nothing. */
export const editorToolKeyAction = (code: string, keys: EditorToolKeys) =>
  ({
    KeyA: keys.onArrowTool,
    KeyB: keys.onToggleBladeTool,
    KeyC: keys.onToggleCrop,
    KeyF: keys.onResizeCanvas,
    KeyK: keys.onToggleKeyboardPanel,
    KeyM: keys.onToggleCursorPanel,
    KeyN: keys.onCounterTool,
    KeyP: keys.onTogglePlayback,
    KeyR: keys.onToggleRangeTool,
    KeyS: keys.onToggleSnap,
    KeyV: keys.onSelectTool,
  })[code] ?? null;

/** The arrow keys as directions, for nudging a layer or stepping the playhead. */
export const arrowDirections = new Map([
  ["ArrowDown", { x: 0, y: 1 }],
  ["ArrowLeft", { x: -1, y: 0 }],
  ["ArrowRight", { x: 1, y: 0 }],
  ["ArrowUp", { x: 0, y: -1 }],
]);
