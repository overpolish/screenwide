// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { EditorKind } from "../types";
import { useEditorWindowShortcuts } from "../use-editor-window-shortcuts";

import { sendToolPanelRequest } from "./tool-panel-store";

/** Apply the editor's input ownership rules before crossing window boundaries. */
export function usePanelShortcuts(workspace: EditorKind) {
  const forward = (code: string, modifiers: KeyboardEventInit = {}) => {
    sendToolPanelRequest(workspace, {
      event: { code, ...modifiers },
      type: "shortcut",
    });
  };
  useEditorWindowShortcuts({
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
    onExport: () => {
      forward("KeyE", { ctrlKey: true });
    },
    onMoveBackward: () => {
      forward("BracketLeft");
    },
    onMoveForward: () => {
      forward("BracketRight");
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
    onRedo: () => {
      forward("KeyZ", { ctrlKey: true, key: "z", shiftKey: true });
    },
    onResizeCanvas: () => {
      forward("KeyF");
    },
    onSelectTool: () => {
      forward("KeyV");
    },
    onToggleBladeTool: () => {
      forward("KeyB");
    },
    onToggleCrop: () => {
      forward("KeyC");
    },
    onTogglePlayback: () => {
      forward("KeyP");
    },
    onToggleRangeTool: () => {
      forward("KeyR");
    },
    onUndo: () => {
      forward("KeyZ", { ctrlKey: true, key: "z" });
    },
  });
}
