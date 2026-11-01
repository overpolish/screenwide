// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const hooks = vi.hoisted(() => ({
  effects: [] as (() => void)[],
  index: 0,
  refs: [] as { current: unknown }[],
}));
vi.mock("react", () => ({
  useEffect: (effect: () => () => void) => hooks.effects.push(effect()),
  useRef: (value: unknown) =>
    (hooks.refs[hooks.index++] ??= { current: value }),
}));
vi.mock("./escape-focus", () => ({
  preserveEscapeFocus: () => () => undefined,
}));
vi.mock("./keyboard-target", () => ({
  editsText: () => false,
  ownsActivationKeys: () => false,
  ownsArrowKeys: () => false,
  ownsPopupInteractionKeys: () => false,
  ownsTextEditingKeys: () => false,
}));

import { useExportIslandKeys } from "./export-island-keys";
import { useEditorWindowShortcuts } from "./use-editor-window-shortcuts";

const press = (code: string, modifiers: { metaKey?: boolean } = {}) => {
  const event = Object.assign(new Event("keydown", { cancelable: true }), {
    altKey: false,
    code,
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    ...modifiers,
  });
  window.dispatchEvent(event);
  return event;
};

beforeEach(() => {
  hooks.refs = [];
  hooks.index = 0;
  vi.stubGlobal("window", new EventTarget());
});
afterEach(() => {
  hooks.effects.splice(0).forEach((dispose) => {
    dispose();
  });
  vi.unstubAllGlobals();
});

describe("editor shortcuts under the Export island", () => {
  // The editor window carries several shortcut hooks side by side, and keys
  // pressed in the island's window are replayed on it, so every one of them
  // has to stand down while the island stands over the editor.
  it("edits nothing, and Escape puts the island away once", () => {
    const onDelete = vi.fn();
    const onDeselect = vi.fn();
    const onTool = vi.fn();
    const putAway = vi.fn();
    useEditorWindowShortcuts({ onDelete, onDeselect });
    useEditorWindowShortcuts({ onDeselect, onTool });
    useExportIslandKeys(putAway);

    press("Delete");
    press("KeyA");
    expect(press("Escape").defaultPrevented).toBe(true);

    expect(putAway).toHaveBeenCalledOnce();
    expect(onDelete).not.toHaveBeenCalled();
    expect(onDeselect).not.toHaveBeenCalled();
    expect(onTool).not.toHaveBeenCalled();
  });

  it("still plays, and Export still answers", () => {
    const onTogglePlayback = vi.fn();
    const onExport = vi.fn();
    useEditorWindowShortcuts({ onExport, onTogglePlayback });
    useExportIslandKeys(vi.fn());

    press("Space");
    press("KeyE", { metaKey: true });

    expect(onTogglePlayback).toHaveBeenCalledOnce();
    expect(onExport).toHaveBeenCalledOnce();
  });

  it("gives the keys back once the island is gone", () => {
    const onDeselect = vi.fn();
    const putAway = vi.fn();
    useEditorWindowShortcuts({ onDeselect });
    useExportIslandKeys(putAway);
    hooks.effects.pop()?.();

    press("Escape");

    expect(putAway).not.toHaveBeenCalled();
    expect(onDeselect).toHaveBeenCalledOnce();
  });
});
