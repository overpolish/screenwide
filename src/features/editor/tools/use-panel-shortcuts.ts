// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEditorWindowShortcuts } from "../shortcuts/use-editor-window-shortcuts";
import { EditorKind } from "../types";

import { sendToolPanelRequest } from "./tool-panel-store";
import { drawingToolKey } from "./tool-registry";

/** Sends a key to the editor that owns `workspace`, as if pressed there. */
export const forwardToolPanelShortcut = (
  workspace: EditorKind,
  code: string,
  modifiers: KeyboardEventInit = {},
) => {
  sendToolPanelRequest(workspace, {
    event: { code, ...modifiers },
    type: "shortcut",
  });
};

/** Apply the editor's input ownership rules before crossing window boundaries. */
export function usePanelShortcuts(workspace: EditorKind) {
  const forward = (code: string, modifiers?: KeyboardEventInit) => {
    forwardToolPanelShortcut(workspace, code, modifiers);
  };
  useEditorWindowShortcuts({
    onArrange: (move) => {
      forward(
        move === "forward" || move === "front" ? "BracketRight" : "BracketLeft",
        { shiftKey: move === "front" || move === "back" },
      );
    },
    onConfirm: () => {
      forward("Enter");
    },
    onCopy: () => {
      forward("KeyC", { ctrlKey: true });
    },
    onCutTimeline: () => {
      forward("KeyB", { ctrlKey: true });
    },
    onDelete: () => {
      forward("Delete");
    },
    onDeselect: () => {
      forward("Escape");
    },
    onDeselectAll: () => {
      forward("KeyD", { ctrlKey: true });
    },
    onExport: () => {
      forward("KeyE", { ctrlKey: true });
    },
    onNudge: (x, y, shiftKey) => {
      forward(
        x < 0
          ? "ArrowLeft"
          : x > 0
            ? "ArrowRight"
            : y < 0
              ? "ArrowUp"
              : "ArrowDown",
        { shiftKey },
      );
    },
    onPaste: () => {
      forward("KeyV", { ctrlKey: true, key: "v" });
    },
    onRedo: () => {
      forward("KeyZ", { ctrlKey: true, key: "z", shiftKey: true });
    },
    onResizeCanvas: () => {
      forward("KeyF");
    },
    onSeekEdge: (edge) => {
      forward(edge === "start" ? "Home" : "End");
    },
    onSelectTool: () => {
      forward("KeyV");
    },
    onToggleBladeTool: () => {
      forward("KeyB");
    },
    onToggleCrop: () => {
      forward("KeyK");
    },
    onTogglePlayback: () => {
      forward("KeyP");
    },
    onToggleRangeTool: () => {
      forward("KeyR", { shiftKey: true });
    },
    onToggleSnap: () => {
      forward("KeyN");
    },
    onTool: (tool) => {
      const { code, modifiers } = drawingToolKey(tool);
      forward(code, modifiers);
    },
    onUndo: () => {
      forward("KeyZ", { ctrlKey: true, key: "z" });
    },
  });
}
