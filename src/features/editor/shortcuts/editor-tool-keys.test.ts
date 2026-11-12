// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it, vi } from "vitest";

import { MODE_SHORTCUTS, TRACKS_SHORTCUT } from "../toolbar/editor-modes";
import {
  ANNOTATION_TOOLS,
  EDITOR_TOOL_SHORTCUTS,
} from "../tools/tool-registry";

import {
  editorCommandKeyAction,
  editorToolKeyAction,
  EditorToolKeys,
} from "./editor-tool-keys";

describe("the editor's tool keys", () => {
  it("reaches every tool that states a letter", () => {
    const keys = {
      onResizeCanvas: vi.fn(),
      onSceneTool: vi.fn(),
      onSelectTool: vi.fn(),
      onToggleCrop: vi.fn(),
      onTool: vi.fn(),
    };
    for (const { id, isShifted, shortcut } of EDITOR_TOOL_SHORTCUTS) {
      const action = editorToolKeyAction(`Key${shortcut}`, keys, isShifted);
      expect(action, `${id} has no action on ${shortcut}`).not.toBeNull();
      action?.();
    }
    // A drawing tool arrives through the one callback, under its own name,
    // and every drawing tool the toolbars show has a letter.
    expect(keys.onTool.mock.calls.flat().sort()).toEqual(
      ANNOTATION_TOOLS.map((tool) => tool.id).sort(),
    );
    expect(keys.onSelectTool).toHaveBeenCalledOnce();
    expect(keys.onToggleCrop).toHaveBeenCalledOnce();
    expect(keys.onResizeCanvas).toHaveBeenCalledOnce();
    expect(keys.onSceneTool).toHaveBeenCalledOnce();
  });

  it("gives no two tools the same key", () => {
    const keys = EDITOR_TOOL_SHORTCUTS.map(
      ({ isShifted, shortcut }) => `${isShifted ? "Shift+" : ""}${shortcut}`,
    );
    expect(new Set(keys).size).toBe(keys.length);
  });

  it("keeps the tools' keys off the keys that belong to no tool", () => {
    // A window that answers no tool hears its letters as the keys below, so
    // a tool given one of them would take it in every window that answers
    // tools, as Focus once took the blade's B.
    const others = {
      onToggleBladeTool: vi.fn(),
      onToggleCameraPanel: vi.fn(),
      onToggleCaptionsPanel: vi.fn(),
      onToggleCursorPanel: vi.fn(),
      onToggleKeyboardPanel: vi.fn(),
      onTogglePlayback: vi.fn(),
      onToggleRangeTool: vi.fn(),
      onToggleScreenPanel: vi.fn(),
      onToggleSnap: vi.fn(),
      onToggleSoundPanel: vi.fn(),
      onToggleTracks: vi.fn(),
      onToggleTranscriptPanel: vi.fn(),
    };
    for (const { id, isShifted, shortcut } of EDITOR_TOOL_SHORTCUTS)
      expect(
        editorToolKeyAction(`Key${shortcut}`, others, isShifted),
        `${id}'s ${shortcut} belongs to another key`,
      ).toBeNull();
  });

  it("opens each mode, and Tracks, by the key its button shows", () => {
    const callbackOf = {
      camera: "onToggleCameraPanel",
      captions: "onToggleCaptionsPanel",
      cursor: "onToggleCursorPanel",
      frame: "onResizeCanvas",
      keyboard: "onToggleKeyboardPanel",
      scene: "onSceneTool",
      screen: "onToggleScreenPanel",
      sound: "onToggleSoundPanel",
      transcript: "onToggleTranscriptPanel",
    } satisfies Record<keyof typeof MODE_SHORTCUTS, keyof EditorToolKeys>;
    const buttons: [string, string, keyof EditorToolKeys][] = [
      ...Object.entries(MODE_SHORTCUTS).map(
        ([mode, shortcut]) =>
          [mode, shortcut, callbackOf[mode as keyof typeof MODE_SHORTCUTS]] as [
            string,
            string,
            keyof EditorToolKeys,
          ],
      ),
      ["tracks", TRACKS_SHORTCUT, "onToggleTracks"],
    ];
    for (const [button, shortcut, callback] of buttons) {
      const keys = { [callback]: vi.fn(), onTool: vi.fn() };
      const isShifted = shortcut.startsWith("Shift+");
      const code = shortcut.replace(/^Shift\+/, "").replace(/^(\w)$/, "Key$1");
      editorToolKeyAction(code, keys, isShifted)?.();
      expect(keys[callback], `${button} on ${shortcut}`).toHaveBeenCalledOnce();
    }
  });

  it("gives a letter with Shift to what stands beside its tool, never the tool", () => {
    const keys = {
      onToggleCrop: vi.fn(),
      onToggleKeyboardPanel: vi.fn(),
      onToggleRangeTool: vi.fn(),
      onTool: vi.fn(),
    };
    editorToolKeyAction("KeyR", keys, true)?.();
    editorToolKeyAction("KeyK", keys, true)?.();
    editorToolKeyAction("KeyO", keys, true)?.();
    expect(keys.onToggleRangeTool).toHaveBeenCalledOnce();
    expect(keys.onToggleKeyboardPanel).toHaveBeenCalledOnce();
    expect(keys.onToggleCrop).not.toHaveBeenCalled();
    // Shift+O is the focus pen's dimmed look, Spotlight.
    expect(keys.onTool.mock.calls).toEqual([["spotlight"]]);
    expect(editorToolKeyAction("KeyA", keys, true)).toBeNull();
  });

  it("leaves cut, copy and paste to a field being typed in, but not Export", () => {
    const keys = {
      onCopy: vi.fn(),
      onCutTimeline: vi.fn(),
      onExport: vi.fn(),
      onPaste: vi.fn(),
    };
    for (const code of ["KeyB", "KeyC", "KeyV"])
      expect(editorCommandKeyAction(code, keys, true)).toBeNull();
    editorCommandKeyAction("KeyE", keys, true)?.();
    editorCommandKeyAction("KeyV", keys, false)?.();
    expect(keys.onExport).toHaveBeenCalledOnce();
    expect(keys.onPaste).toHaveBeenCalledOnce();
  });
});
