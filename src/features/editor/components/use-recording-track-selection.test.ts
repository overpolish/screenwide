// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
import { expect, it, vi } from "vitest";

import { useRecordingTrackSelection } from "./use-recording-track-selection";
const mocks = vi.hoisted(() => ({
  bounds: { height: 300, width: 400, x: 0, y: 0 },
  open: vi.fn(),
}));
vi.mock("react", () => ({ useCallback: (callback: unknown) => callback }));
vi.mock("../tool-panels/use-tool-panel", () => ({
  previewViewport: () => ({ getBoundingClientRect: () => mocks.bounds }),
  useToolPanel: () => ({ openPanel: mocks.open }),
}));
it("activates Select and opens its panel on every track selection", () => {
  const setTool = vi.fn();
  const clear = vi.fn();
  const selected = vi.fn();
  const choose = useRecordingTrackSelection({
    clearAnnotations: { current: clear },
    clearKeyboard: clear,
    onSelectedTrackChange: selected,
    setTool,
  });
  choose("primary");
  choose("primary");
  expect(setTool.mock.calls).toEqual([["select"], ["select"]]);
  expect(mocks.open).toHaveBeenCalledTimes(2);
  expect(mocks.open).toHaveBeenLastCalledWith("selection", mocks.bounds, false);
});
