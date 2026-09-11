// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const hooks = vi.hoisted(() => ({
  activation: false,
  arrows: false,
  effects: [] as (() => void)[],
  index: 0,
  popup: false,
  refs: [] as { current: unknown }[],
  send: vi.fn(),
  text: false,
}));
vi.mock("react", () => ({
  useEffect: (effect: () => () => void) => hooks.effects.push(effect()),
  useRef: (value: unknown) =>
    (hooks.refs[hooks.index++] ??= { current: value }),
}));
vi.mock("../escape-focus", () => ({
  preserveEscapeFocus: () => () => undefined,
}));
vi.mock("../keyboard-target", () => ({
  ownsActivationKeys: () => hooks.activation,
  ownsArrowKeys: () => hooks.arrows,
  ownsPopupInteractionKeys: () => hooks.popup,
  ownsTextEditingKeys: () => hooks.text,
}));
vi.mock("./tool-panel-store", () => ({ sendToolPanelRequest: hooks.send }));

import { useEditorWindowShortcuts } from "../use-editor-window-shortcuts";

import { usePanelShortcuts } from "./use-panel-shortcuts";

const key = (code: string, values = {}, type = "keydown") => {
  const event = Object.assign(new Event(type, { cancelable: true }), {
    code,
    ...values,
  });
  const stop = vi.spyOn(event, "stopPropagation");
  window.dispatchEvent(event);
  return { event, stop };
};

beforeEach(() => {
  hooks.refs = [];
  hooks.index = 0;
  hooks.text = hooks.arrows = hooks.activation = hooks.popup = false;
  hooks.send.mockClear();
  vi.stubGlobal("window", new EventTarget());
});
afterEach(() => {
  hooks.effects.splice(0).forEach((dispose) => {
    dispose();
  });
  vi.unstubAllGlobals();
});

describe("panel editor shortcuts", () => {
  it("forwards V once and consumes both press and release before navigation", () => {
    usePanelShortcuts("recording");
    expect(key("KeyV").stop).toHaveBeenCalledOnce();
    expect(key("KeyV", { repeat: true }).stop).toHaveBeenCalledOnce();
    expect(key("KeyV", {}, "keyup").stop).toHaveBeenCalledOnce();
    expect(hooks.send).toHaveBeenCalledExactlyOnceWith("recording", {
      event: { code: "KeyV" },
      type: "shortcut",
    });
  });

  it("preserves text editing and composition", () => {
    usePanelShortcuts("recording");
    hooks.text = true;
    expect(key("KeyV").event.defaultPrevented).toBe(false);
    hooks.text = false;
    key("KeyV", { isComposing: true });
    expect(hooks.send).not.toHaveBeenCalled();
  });

  it("leaves Tab, widget arrows and keyboard activation with panel controls", () => {
    usePanelShortcuts("recording");
    expect(key("Tab").stop).not.toHaveBeenCalled();
    hooks.arrows = hooks.activation = true;
    expect(key("ArrowRight").stop).not.toHaveBeenCalled();
    expect(key("Space").stop).not.toHaveBeenCalled();
    expect(hooks.send).not.toHaveBeenCalled();
  });

  it("forwards canvas arrow intent and modifiers outside widgets", () => {
    usePanelShortcuts("screenshot");
    key("ArrowLeft", { shiftKey: true });
    key("ArrowLeft", { repeat: true, shiftKey: true });
    expect(hooks.send).toHaveBeenCalledTimes(2);
    expect(hooks.send).toHaveBeenCalledWith("screenshot", {
      event: { code: "ArrowLeft", shiftKey: true },
      type: "shortcut",
    });
  });

  it("forwards undo and redo while keeping text-field history local", () => {
    usePanelShortcuts("recording");
    key("KeyZ", { metaKey: true });
    key("KeyZ", { metaKey: true, shiftKey: true });
    expect(hooks.send).toHaveBeenCalledTimes(2);
    expect(hooks.send).toHaveBeenLastCalledWith("recording", {
      event: { code: "KeyZ", ctrlKey: true, key: "z", shiftKey: true },
      type: "shortcut",
    });
    hooks.text = true;
    key("KeyZ", { metaKey: true });
    expect(hooks.send).toHaveBeenCalledTimes(2);
  });

  it("keeps release suppression through a render caused by the shortcut", () => {
    const select = vi.fn();
    useEditorWindowShortcuts({ onSelectTool: select });
    key("KeyV");
    hooks.effects.splice(0).forEach((dispose) => {
      dispose();
    });
    hooks.index = 0;
    useEditorWindowShortcuts({ onSelectTool: select });
    expect(key("KeyV", {}, "keyup").stop).toHaveBeenCalledOnce();
    expect(select).toHaveBeenCalledOnce();
  });
});
