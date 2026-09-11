// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { afterEach, describe, expect, it, vi } from "vitest";

import { growEditorForPanel } from "../api";

import {
  openToolPanelSpace,
  panelGrowthDelta,
  toolPanelGutter,
} from "./tool-panel-space";

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

describe("panel growth target", () => {
  it("makes enough room from minimum width in one activation", () => {
    const width = 400;
    const fitWidth = 960;
    const growth = panelGrowthDelta(width, fitWidth, 320);
    expect(growth).toBe(880);
    expect(width + growth - 320).toBe(fitWidth);
  });

  it("adds just the gutter when the full-height preview already fits", () => {
    expect(panelGrowthDelta(1200, 960, 320)).toBe(320);
  });

  it.each([NaN, 0, -1, Infinity])(
    "falls back to gutter for unavailable geometry %s",
    (width) => {
      expect(panelGrowthDelta(400, width, 320)).toBe(320);
    },
  );
});
