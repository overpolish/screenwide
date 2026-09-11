// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { afterEach, beforeEach, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  active: true,
  cleanup: [] as (() => void)[],
  fit: vi.fn(),
  move: vi.fn(() => Promise.resolve()),
  resize: (): void => undefined,
  setBasis: vi.fn(),
  width: 1200,
}));
vi.mock("react", () => ({
  useEffect: (effect: () => (() => void) | undefined) => {
    const cleanup = effect();
    if (cleanup) mocks.cleanup.push(cleanup);
  },
}));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ label: "editor" }),
}));
vi.mock("../../popup-panel/api", () => ({ movePopupPanel: mocks.move }));
vi.mock("../../popup-panel/store", () => ({
  activePopupPanel: (
    state: { active: Record<string, unknown> },
    panel: string,
  ) => state.active[panel] ?? null,
  usePopupPanelStore: (select: (state: unknown) => unknown) =>
    select({
      active: {
        // The other workspace's panel is open throughout: this one follows
        // its own window and nothing else.
        "tool-panel-recording": mocks.active
          ? { content: { kind: "tool", tool: "cursor" } }
          : null,
        "tool-panel-screenshot": { content: { kind: "tool", tool: "cursor" } },
      },
    }),
}));
vi.mock("../components/preview-fit-context", () => ({
  usePreviewFit: () => ({ fitPreview: mocks.fit, setFitBasis: mocks.setBasis }),
}));
vi.mock("./tool-panel-space", () => ({ toolPanelGutter: 320 }));
vi.mock("./use-tool-panel", () => ({
  panelOffset: (bounds: unknown) => bounds,
  previewViewport: () => ({
    getBoundingClientRect: () => ({ width: mocks.width }),
  }),
}));

import { ToolPanelPlacement } from "./tool-panel-placement";

beforeEach(() => {
  mocks.active = true;
  mocks.width = 1200;
  vi.clearAllMocks();
  vi.stubGlobal(
    "ResizeObserver",
    class {
      constructor(callback: () => void) {
        mocks.resize = callback;
      }
      disconnect() {}
      observe() {}
    },
  );
  vi.stubGlobal("requestAnimationFrame", (callback: () => void) => {
    callback();
    return 1;
  });
  vi.stubGlobal("cancelAnimationFrame", vi.fn());
});
afterEach(() => {
  mocks.cleanup.splice(0).forEach((cleanup) => {
    cleanup();
  });
  vi.unstubAllGlobals();
});

it("updates the reset destination after narrowing without moving the current view", () => {
  ToolPanelPlacement({ workspace: "recording" });
  mocks.resize();
  expect(mocks.setBasis).toHaveBeenLastCalledWith(880);
  mocks.width = 600;
  mocks.resize();
  expect(mocks.setBasis).toHaveBeenLastCalledWith(280);
  // The move names this workspace's own panel window, never the other's.
  expect(mocks.move).toHaveBeenNthCalledWith(
    1,
    "editor",
    { width: 1200 },
    "tool-panel-recording",
  );
  expect(mocks.fit).not.toHaveBeenCalled();
});

it("returns reset to the full viewport after dismissal without resetting the view", () => {
  mocks.active = false;
  ToolPanelPlacement({ workspace: "recording" });
  expect(mocks.setBasis).toHaveBeenCalledExactlyOnceWith();
  expect(mocks.fit).not.toHaveBeenCalled();
});
