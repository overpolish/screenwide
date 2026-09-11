// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { afterEach, beforeEach, expect, it, vi } from "vitest";

const state = vi.hoisted(() => ({
  content: null as { kind: "tool"; workspace: string } | null,
  dispose: (): void => undefined,
  listener: (): void => undefined,
  set: vi.fn(),
}));
vi.mock("react", () => ({
  useEffect: (effect: () => () => void) => {
    state.dispose = effect();
  },
  useState: (initial: () => unknown) => [initial(), state.set],
}));
vi.mock("../../popup-panel/store", () => ({
  usePopupPanelStore: {
    getState: () => ({ active: { content: state.content } }),
    subscribe: (listener: () => void) => {
      state.listener = listener;
      return () => {
        state.listener = () => undefined;
      };
    },
  },
}));

import { useCanvasTool } from "./use-canvas-tool";

beforeEach(() => {
  state.content = null;
  state.set.mockClear();
});
afterEach(() => {
  state.dispose();
});

it("retires Select when Cursor opens and never restores it on close", () => {
  expect(useCanvasTool("recording", "select")[0]).toBe("select");
  state.content = { kind: "tool", workspace: "recording" };
  state.listener();
  expect(state.set).toHaveBeenCalledExactlyOnceWith(null);
  state.content = null;
  state.listener();
  expect(state.set).toHaveBeenCalledOnce();
});

it("starts with no canvas tool if its panel is already open", () => {
  state.content = { kind: "tool", workspace: "recording" };
  expect(useCanvasTool("recording", "select")[0]).toBeNull();
});

it("ignores panels belonging to the other workspace", () => {
  useCanvasTool("screenshot", "select");
  state.content = { kind: "tool", workspace: "recording" };
  state.listener();
  expect(state.set).not.toHaveBeenCalled();
});
