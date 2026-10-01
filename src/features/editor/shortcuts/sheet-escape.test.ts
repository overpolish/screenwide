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
  ownsActivationKeys: () => false,
  ownsArrowKeys: () => false,
  ownsPopupInteractionKeys: () => false,
  ownsTextEditingKeys: () => false,
}));

import { useSheetEscape } from "./sheet-escape";
import { useEditorWindowShortcuts } from "./use-editor-window-shortcuts";

const press = (code: string) => {
  const event = Object.assign(new Event("keydown", { cancelable: true }), {
    code,
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

describe("editor shortcuts under a sheet", () => {
  // The editor window carries several shortcut hooks side by side, and keys
  // pressed in the tool panel are replayed on it, so every one of them has to
  // stand down while a sheet is up.
  it("answers no shortcut, and Escape closes the sheet once", () => {
    const onDelete = vi.fn();
    const onDeselect = vi.fn();
    const onTogglePlayback = vi.fn();
    const dismiss = vi.fn();
    useEditorWindowShortcuts({ onDelete, onDeselect });
    useEditorWindowShortcuts({ onDeselect, onTogglePlayback });
    useSheetEscape(dismiss);

    press("Delete");
    press("KeyP");
    expect(press("Escape").defaultPrevented).toBe(true);

    expect(dismiss).toHaveBeenCalledOnce();
    expect(onDelete).not.toHaveBeenCalled();
    expect(onDeselect).not.toHaveBeenCalled();
    expect(onTogglePlayback).not.toHaveBeenCalled();
  });

  it("gives the keys back once the sheet is gone", () => {
    const onDeselect = vi.fn();
    const dismiss = vi.fn();
    useEditorWindowShortcuts({ onDeselect });
    useSheetEscape(dismiss);
    hooks.effects.pop()?.();

    press("Escape");

    expect(dismiss).not.toHaveBeenCalled();
    expect(onDeselect).toHaveBeenCalledOnce();
  });
});
