// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it, vi } from "vitest";

import {
  ANNOTATION_TOOLS,
  EDITOR_TOOL_SHORTCUTS,
} from "../tool-panels/tool-registry";

import { editorToolKeyAction } from "./editor-tool-keys";

describe("the editor's tool keys", () => {
  it("reaches every tool that states a letter", () => {
    const keys = {
      onMarqueeTool: vi.fn(),
      onResizeCanvas: vi.fn(),
      onSceneTool: vi.fn(),
      onSelectTool: vi.fn(),
      onToggleCrop: vi.fn(),
      onTool: vi.fn(),
    };
    for (const { id, shortcut } of EDITOR_TOOL_SHORTCUTS) {
      const action = editorToolKeyAction(`Key${shortcut}`, keys);
      expect(action, `${id} has no action on ${shortcut}`).not.toBeNull();
      action?.();
    }
    // A drawing tool arrives through the one callback, under its own name.
    expect(keys.onTool.mock.calls.flat()).toEqual(
      ANNOTATION_TOOLS.map((tool) => tool.id),
    );
    expect(keys.onMarqueeTool).toHaveBeenCalledOnce();
    expect(keys.onSelectTool).toHaveBeenCalledOnce();
    expect(keys.onToggleCrop).toHaveBeenCalledOnce();
    expect(keys.onResizeCanvas).toHaveBeenCalledOnce();
    expect(keys.onSceneTool).toHaveBeenCalledOnce();
  });

  it("gives no two tools the same letter", () => {
    const letters = EDITOR_TOOL_SHORTCUTS.map((tool) => tool.shortcut);
    expect(new Set(letters).size).toBe(letters.length);
  });

  it("keeps the tools' letters off the keys that belong to no tool", () => {
    // A tool's letter is answered before the rest of the table, so a tool
    // that took one of these would silently steal it.
    const others = {
      onToggleBladeTool: vi.fn(),
      onToggleCursorPanel: vi.fn(),
      onToggleKeyboardPanel: vi.fn(),
      onTogglePlayback: vi.fn(),
      onToggleRangeTool: vi.fn(),
      onToggleSnap: vi.fn(),
    };
    for (const { shortcut } of EDITOR_TOOL_SHORTCUTS)
      expect(editorToolKeyAction(`Key${shortcut}`, others)).toBeNull();
  });

  it("gives a letter with Shift to the setting beside the tool, never the tool", () => {
    const keys = {
      onToggleCrop: vi.fn(),
      onToggleCursorPanel: vi.fn(),
      onToggleRangeTool: vi.fn(),
      onToggleSnap: vi.fn(),
      onTool: vi.fn(),
    };
    editorToolKeyAction("KeyS", keys, true)?.();
    editorToolKeyAction("KeyR", keys, true)?.();
    editorToolKeyAction("KeyC", keys, true)?.();
    expect(keys.onToggleSnap).toHaveBeenCalledOnce();
    expect(keys.onToggleRangeTool).toHaveBeenCalledOnce();
    expect(keys.onToggleCursorPanel).toHaveBeenCalledOnce();
    expect(keys.onToggleCrop).not.toHaveBeenCalled();
    expect(keys.onTool).not.toHaveBeenCalled();
    expect(editorToolKeyAction("KeyA", keys, true)).toBeNull();
  });
});
