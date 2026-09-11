// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  active: null as {
    kind: "tool";
    tool: "cursor" | "selection";
    workspace: "recording" | "screenshot";
  } | null,
  close: vi.fn(() => Promise.resolve()),
  fitPreview: vi.fn(),
  openPanel: vi.fn(() => Promise.resolve()),
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-assertion
  panelByTool: {} as Record<string, "cursor" | "selection" | undefined>,
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-assertion
  resetByTool: {} as Record<string, boolean>,
  // The hook holds one ref, so a single cell stands in for React's own.
  settled: { current: undefined },
  viewport: { getBoundingClientRect: () => ({ top: 0, width: 800 }) },
}));

vi.mock("react", () => ({
  useEffect: (effect: () => (() => void) | undefined) => {
    effect();
  },
  useRef: () => mocks.settled,
}));

vi.mock("../../popup-panel/store", () => ({
  activePopupPanel: (
    state: { active: Record<string, unknown> },
    panel: string,
  ) => state.active[panel] ?? null,
  usePopupPanelStore: {
    getState: () => ({
      active: {
        // Only this workspace's own panel window is ever consulted, so the
        // other editor's open panel stands here throughout.
        "tool-panel-recording": { content: mocks.active },
        "tool-panel-screenshot": {
          content: { kind: "tool", tool: "cursor", workspace: "screenshot" },
        },
      },
    }),
  },
}));

vi.mock("../components/preview-fit-context", () => ({
  usePreviewFit: () => ({ fitPreview: mocks.fitPreview }),
}));

vi.mock("./tool-registry", () => ({
  toolPanel: (tool: string) => mocks.panelByTool[tool],
  toolResetsView: (tool: string) => mocks.resetByTool[tool] ?? false,
}));

vi.mock("./use-tool-panel", () => ({
  previewViewport: () => mocks.viewport,
  useToolPanel: () => ({ close: mocks.close, openPanel: mocks.openPanel }),
}));

import { useToolPanelFollowsTool } from "./use-tool-panel-follows-tool";

type ToolInHand = Parameters<typeof useToolPanelFollowsTool>[1];

/** One render of a workspace with `tool` in hand, settled. */
const follow = async (tool: ToolInHand) => {
  // eslint-disable-next-line @eslint-react/rules-of-hooks
  useToolPanelFollowsTool("recording", tool);
  await new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
};

beforeEach(() => {
  mocks.active = null;
  mocks.panelByTool = { cursor: "cursor", select: "selection" };
  mocks.resetByTool = { crop: true, cursor: true, select: false };
  mocks.settled = { current: undefined };
  vi.clearAllMocks();
  vi.stubGlobal("document", { querySelector: () => null });
});

describe("useToolPanelFollowsTool", () => {
  it("opens the selection panel for the tool a session starts in", async () => {
    await follow("select");

    expect(mocks.openPanel).toHaveBeenCalledExactlyOnceWith(
      "selection",
      mocks.viewport.getBoundingClientRect(),
      false,
    );
    expect(mocks.close).not.toHaveBeenCalled();
  });

  it("anchors the panel on the tool's own button when the toolbar shows one", async () => {
    const bounds = { top: 4, width: 32 };
    vi.stubGlobal("document", {
      querySelector: (selector: string) =>
        selector === '[data-editor-tool="select"]'
          ? { getBoundingClientRect: () => bounds }
          : null,
    });

    await follow("select");

    expect(mocks.openPanel).toHaveBeenCalledWith("selection", bounds, false);
  });

  it("swaps the panel in place when another tool with one is taken up", async () => {
    await follow("select");
    mocks.active = { kind: "tool", tool: "selection", workspace: "recording" };
    await follow("cursor");

    expect(mocks.openPanel).toHaveBeenLastCalledWith(
      "cursor",
      mocks.viewport.getBoundingClientRect(),
      true,
    );
    expect(mocks.close).not.toHaveBeenCalled();
  });

  it("closes the panel for a tool without one, applying its fit policy", async () => {
    await follow("select");
    mocks.active = { kind: "tool", tool: "selection", workspace: "recording" };
    await follow("crop");

    expect(mocks.close).toHaveBeenCalledOnce();
    expect(mocks.fitPreview).toHaveBeenCalledExactlyOnceWith();
  });

  it("closes its own panel when the tool is put away", async () => {
    await follow("select");
    mocks.active = { kind: "tool", tool: "selection", workspace: "recording" };
    await follow(null);

    expect(mocks.close).toHaveBeenCalledOnce();
    expect(mocks.fitPreview).not.toHaveBeenCalled();
  });

  it("leaves a panel-only tool's panel alone when it retires the canvas tool", async () => {
    await follow("select");
    mocks.active = { kind: "tool", tool: "cursor", workspace: "recording" };
    await follow(null);

    expect(mocks.close).not.toHaveBeenCalled();
  });

  it("settles once per tool, however often the workspace renders", async () => {
    await follow("select");
    mocks.active = { kind: "tool", tool: "selection", workspace: "recording" };
    await follow("select");

    expect(mocks.openPanel).toHaveBeenCalledOnce();
  });
});
