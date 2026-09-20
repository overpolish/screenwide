// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  editorToolForShortcut,
  EditorToolId,
  isDrawingTool,
} from "./tool-panels/tool-registry";

import type { AnnotationKind } from "../../components/shared/annotation-style/types";

/**
 * The plain letter keys that take up a tool or put its panel on screen.
 *
 * One table rather than a chain of conditions, so a tool added later states
 * its key in the same place as the rest and the hook that owns the window's
 * keyboard keeps the shape it has: the guards - a modifier, a repeat, a field
 * being typed in - are applied once, around the whole table.
 *
 * A tool's own letter comes from `EDITOR_TOOLS`, so a drawing tool added
 * there is reachable from the keyboard without being named again here. The
 * keys below belong to no tool.
 */
export type EditorToolKeys = {
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
  /** A drawing tool's letter, taking the tool up: A for the arrow, N for the
   * counter. One callback for every drawing tool, because taking one up is
   * the same act whichever shape it draws. */
  onTool?: (tool: AnnotationKind) => void;
};

/** What taking up `tool` does in this window. A drawing tool goes through the
 * one `onTool` callback; the others mean something different enough in each
 * workspace to keep a callback of their own. */
const toolKeyAction = (id: EditorToolId, keys: EditorToolKeys) => {
  if (isDrawingTool(id)) {
    const onTool = keys.onTool;
    return (
      onTool &&
      (() => {
        onTool(id);
      })
    );
  }
  return {
    crop: keys.onToggleCrop,
    cursor: undefined,
    frame: keys.onResizeCanvas,
    keyboard: undefined,
    select: keys.onSelectTool,
  }[id];
};

/** What `code` does in this window, or null where it does nothing. */
export const editorToolKeyAction = (code: string, keys: EditorToolKeys) => {
  const tool = code.startsWith("Key")
    ? editorToolForShortcut(code.slice(3))
    : null;
  if (tool !== null) return toolKeyAction(tool, keys) ?? null;
  return (
    {
      KeyB: keys.onToggleBladeTool,
      KeyK: keys.onToggleKeyboardPanel,
      KeyM: keys.onToggleCursorPanel,
      KeyP: keys.onTogglePlayback,
      KeyR: keys.onToggleRangeTool,
      KeyS: keys.onToggleSnap,
    }[code] ?? null
  );
};

/** The arrow keys as directions, for nudging a layer or stepping the playhead. */
export const arrowDirections = new Map([
  ["ArrowDown", { x: 0, y: 1 }],
  ["ArrowLeft", { x: -1, y: 0 }],
  ["ArrowRight", { x: 1, y: 0 }],
  ["ArrowUp", { x: 0, y: -1 }],
]);
