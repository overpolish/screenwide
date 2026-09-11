// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { afterEach, beforeEach, expect, it, vi } from "vitest";

const state = vi.hoisted(() => ({
  content: null as {
    kind: "tool";
    tool: "cursor" | "frame" | "selection";
    workspace: string;
  } | null,
  dispose: (): void => undefined,
  listener: (): void => undefined,
  set: vi.fn(),
  /** Which workspace's panel window the open panel belongs to. */
  workspace: "recording",
}));
vi.mock("react", () => ({
  useEffect: (effect: () => () => void) => {
    state.dispose = effect();
  },
  useState: (initial: () => unknown) => [initial(), state.set],
}));
vi.mock("../../popup-panel/store", () => ({
  activePopupPanel: (
    state: { active: Record<string, unknown> },
    panel: string,
  ) => state.active[panel] ?? null,
  usePopupPanelStore: {
    getState: () => ({
      active: { [`tool-panel-${state.workspace}`]: { content: state.content } },
    }),
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
  state.workspace = "recording";
  state.set.mockClear();
});
afterEach(() => {
  state.dispose();
});

it("retires Select when Cursor opens and never restores it on close", () => {
  expect(useCanvasTool("recording", "select")[0]).toBe("select");
  state.content = { kind: "tool", tool: "cursor", workspace: "recording" };
  state.listener();
  expect(state.set).toHaveBeenCalledExactlyOnceWith(null);
  state.content = null;
  state.listener();
  expect(state.set).toHaveBeenCalledOnce();
});

it("starts with no canvas tool if its panel is already open", () => {
  state.content = { kind: "tool", tool: "cursor", workspace: "recording" };
  expect(useCanvasTool("recording", "select")[0]).toBeNull();
});

it("ignores panels belonging to the other workspace", () => {
  useCanvasTool("screenshot", "select");
  state.content = { kind: "tool", tool: "cursor", workspace: "recording" };
  state.listener();
  expect(state.set).not.toHaveBeenCalled();
});

it("reads its own workspace's panel window, not whichever opened last", () => {
  state.workspace = "screenshot";
  state.content = { kind: "tool", tool: "cursor", workspace: "screenshot" };
  expect(useCanvasTool("recording", "select")[0]).toBe("select");
  expect(useCanvasTool("screenshot", "select")[0]).toBeNull();
});

it("keeps Select in hand while its own panel is open", () => {
  expect(useCanvasTool("recording", "select")[0]).toBe("select");
  state.content = { kind: "tool", tool: "selection", workspace: "recording" };
  state.listener();
  expect(state.set).not.toHaveBeenCalled();
  expect(useCanvasTool("recording", "select")[0]).toBe("select");
});

it("keeps the canvas tool in hand while the Frame panel is open", () => {
  expect(useCanvasTool("screenshot", "canvas")[0]).toBe("canvas");
  state.workspace = "screenshot";
  state.content = { kind: "tool", tool: "frame", workspace: "screenshot" };
  state.listener();
  expect(state.set).not.toHaveBeenCalled();
  expect(useCanvasTool("screenshot", "canvas")[0]).toBe("canvas");
});
