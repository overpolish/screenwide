// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { afterEach, describe, expect, it, vi } from "vitest";

import { growEditorForPanel } from "../api";

import { openToolPanelSpace, toolPanelGutter } from "./tool-panel-space";

const windowMock = vi.hoisted(() => ({
  onResized: vi.fn(),
  unlisten: vi.fn(),
}));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    label: "editor",
    onResized: windowMock.onResized,
  }),
}));
vi.mock("../api", () => ({ growEditorForPanel: vi.fn() }));

afterEach(() => {
  vi.resetAllMocks();
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

describe("one-time panel growth", () => {
  it("does not resize a window that already has room", async () => {
    await openToolPanelSpace(0);
    expect(growEditorForPanel).not.toHaveBeenCalled();
    expect(windowMock.onResized).not.toHaveBeenCalled();
  });

  it.each([true, false])(
    "settles growth=%s and releases its resize listener",
    async (grew) => {
      vi.useFakeTimers();
      windowMock.onResized.mockResolvedValue(windowMock.unlisten);
      vi.mocked(growEditorForPanel).mockResolvedValue(grew);
      vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) =>
        setTimeout(() => {
          callback(0);
        }, 16),
      );
      const opening = openToolPanelSpace(toolPanelGutter);
      await vi.runAllTimersAsync();
      await opening;
      expect(growEditorForPanel).toHaveBeenCalledWith(
        "editor",
        toolPanelGutter,
      );
      expect(windowMock.unlisten).toHaveBeenCalledOnce();
      expect(vi.getTimerCount()).toBe(0);
    },
  );
});
