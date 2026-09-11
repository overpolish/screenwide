// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

type OpenToolPanel = {
  content: {
    kind: "tool";
    tool: "cursor" | "keyboard" | "selection";
    workspace: "recording" | "screenshot";
  };
  id: string;
} | null;

const mocks = vi.hoisted(() => ({
  active: null as OpenToolPanel,
  close: vi.fn(),
  fitPreview: vi.fn(),
  fitWidth: "",
  getBoundingClientRect: vi.fn(),
  getCurrentWindow: vi.fn(() => ({ label: "editor", onResized: vi.fn() })),
  hidePopupPanel: vi.fn(() => Promise.resolve()),
  open: vi.fn(),
  openToolPanelSpace: vi.fn(() => Promise.resolve()),
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-assertion
  panelByTool: {} as Record<string, "cursor" | "selection" | undefined>,
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-assertion
  resetByTool: {} as Record<string, boolean>,
  screenshotActive: null as OpenToolPanel,
  setFitBasis: vi.fn(),
  showPopupPanel: vi.fn(() => Promise.resolve()),
}));

vi.mock("react", () => ({
  useCallback: (callback: unknown) => callback,
}));

vi.mock("@tauri-apps/api/window", () => ({
  LogicalPosition: class LogicalPosition {
    constructor(
      public x: number,
      public y: number,
    ) {}
  },
  LogicalSize: class LogicalSize {
    constructor(
      public width: number,
      public height: number,
    ) {}
  },
  getCurrentWindow: mocks.getCurrentWindow,
}));

vi.mock("../../popup-panel/api", () => ({
  hidePopupPanel: mocks.hidePopupPanel,
  showPopupPanel: mocks.showPopupPanel,
}));

const panels = () => ({
  "tool-panel-recording": mocks.active,
  "tool-panel-screenshot": mocks.screenshotActive,
});

vi.mock("../../popup-panel/store", () => ({
  activePopupPanel: (
    state: { active: Record<string, unknown> },
    panel: string,
  ) => state.active[panel] ?? null,
  usePopupPanelStore: Object.assign(
    (selector: (state: unknown) => unknown) => selector({ active: panels() }),
    {
      getState: () => ({
        active: panels(),
        close: mocks.close,
        open: mocks.open,
      }),
    },
  ),
}));

vi.mock("../components/preview-fit-context", () => ({
  usePreviewFit: () => ({
    fitPreview: mocks.fitPreview,
    setFitBasis: mocks.setFitBasis,
  }),
}));

vi.mock("../api", () => ({ growEditorForPanel: vi.fn() }));

vi.mock("./tool-panel-space", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./tool-panel-space")>()),
  openToolPanelSpace: mocks.openToolPanelSpace,
  toolPanelGutter: 320,
}));

vi.mock("./tool-registry", () => ({
  toolPanel: (tool: string) => mocks.panelByTool[tool],
  toolResetsView: (tool: string) => mocks.resetByTool[tool] ?? false,
}));

import { useToolPanel } from "./use-tool-panel";

/** The recording workspace with its cursor panel up. */
const cursorPanelOpen: OpenToolPanel = {
  content: { kind: "tool", tool: "cursor", workspace: "recording" },
  id: "tool:cursor",
};

const anchor = {
  height: 32,
  left: 20,
  top: 10,
  width: 32,
} as DOMRect;

const setViewportWidth = (width: number) => {
  mocks.getBoundingClientRect.mockReturnValue({
    bottom: 600,
    left: 0,
    right: width,
    top: 0,
    width,
  });
  vi.stubGlobal("document", {
    querySelector: vi.fn(() => ({
      dataset: { previewFitWidth: mocks.fitWidth },
      getBoundingClientRect: mocks.getBoundingClientRect,
    })),
  });
};

const useCursorPanel = () => {
  mocks.panelByTool.cursor = "cursor";
  mocks.resetByTool.cursor = true;
  const panel = useToolPanel("recording");
  return panel.toggle("cursor", anchor);
};

beforeEach(() => {
  mocks.active = null;
  mocks.screenshotActive = null;
  mocks.fitWidth = "";
  mocks.panelByTool = {};
  mocks.resetByTool = {};
  setViewportWidth(800);
  mocks.openToolPanelSpace.mockImplementation(() => Promise.resolve());
  vi.clearAllMocks();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("useToolPanel", () => {
  it("grows a narrow viewport to the complete panel target before fitting", async () => {
    mocks.fitWidth = "960";
    setViewportWidth(400);
    mocks.openToolPanelSpace.mockImplementation(() => {
      setViewportWidth(1280);
      return Promise.resolve();
    });
    await useCursorPanel();
    expect(mocks.openToolPanelSpace).toHaveBeenCalledExactlyOnceWith(880);
    expect(mocks.fitPreview).toHaveBeenCalledExactlyOnceWith(960);
  });

  it("fits and grows once when an opted-in panel opens", async () => {
    mocks.openToolPanelSpace.mockImplementation(() => {
      setViewportWidth(1220);
      return Promise.resolve();
    });
    await useCursorPanel();

    expect(mocks.openToolPanelSpace).toHaveBeenCalledWith(320);
    expect(mocks.fitPreview).toHaveBeenCalledWith(900);
    expect(mocks.open).toHaveBeenCalledOnce();
    expect(mocks.showPopupPanel).toHaveBeenCalledOnce();
  });

  it("requests growth again after closing and reopening at the previous fitted width", async () => {
    mocks.openToolPanelSpace.mockImplementation(() => {
      setViewportWidth(1120);
      return Promise.resolve();
    });
    await useCursorPanel();
    mocks.active = cursorPanelOpen;
    await useToolPanel("recording").toggle("cursor", anchor);
    expect(mocks.openToolPanelSpace).toHaveBeenCalledOnce();
    mocks.active = null;
    await useCursorPanel();
    expect(mocks.openToolPanelSpace).toHaveBeenCalledTimes(2);
    expect(mocks.openToolPanelSpace).toHaveBeenLastCalledWith(320);
  });

  it("opens an overlay panel without growing or fitting", async () => {
    mocks.panelByTool.select = "cursor";
    mocks.resetByTool.cursor = true;
    mocks.resetByTool.select = false;
    const panel = useToolPanel("recording");

    await panel.toggle("select", anchor);

    expect(mocks.openToolPanelSpace).not.toHaveBeenCalled();
    expect(mocks.fitPreview).not.toHaveBeenCalled();
    expect(mocks.open).toHaveBeenCalledOnce();
  });

  it("closes an open panel without changing the view", async () => {
    mocks.active = cursorPanelOpen;
    const panel = useToolPanel("recording");

    await panel.toggle("cursor", anchor);

    expect(mocks.close).toHaveBeenCalledOnce();
    expect(mocks.hidePopupPanel).toHaveBeenCalledOnce();
    expect(mocks.fitPreview).not.toHaveBeenCalled();
    expect(mocks.openToolPanelSpace).not.toHaveBeenCalled();
  });

  it("hands the reset basis back to the full viewport on closing", async () => {
    mocks.active = cursorPanelOpen;
    mocks.panelByTool.cursor = "cursor";
    mocks.resetByTool.cursor = true;

    await useToolPanel("recording").toggle("cursor", anchor);

    expect(mocks.setFitBasis).toHaveBeenCalledOnce();
    expect(mocks.setFitBasis).toHaveBeenCalledWith();
    expect(mocks.fitPreview).not.toHaveBeenCalled();
  });

  it("leaves the panel's own basis alone while it opens", async () => {
    mocks.openToolPanelSpace.mockImplementation(() => {
      setViewportWidth(1220);
      return Promise.resolve();
    });

    await useCursorPanel();

    expect(mocks.fitPreview).toHaveBeenCalledWith(900);
    expect(mocks.setFitBasis).not.toHaveBeenCalled();
  });

  it("switches tools according to the destination policy", async () => {
    mocks.active = {
      content: { kind: "tool", tool: "keyboard", workspace: "recording" },
      id: "tool:keyboard",
    };
    mocks.panelByTool.cursor = "cursor";
    mocks.resetByTool.cursor = true;
    const panel = useToolPanel("recording");

    await panel.toggle("cursor", anchor);
    expect(mocks.fitPreview).toHaveBeenCalledWith(480);
    expect(mocks.open).toHaveBeenCalledOnce();
    expect(mocks.showPopupPanel).toHaveBeenCalledOnce();
    expect(mocks.hidePopupPanel).not.toHaveBeenCalled();
  });

  it("opens in its own workspace's panel window, leaving the other alone", async () => {
    mocks.active = cursorPanelOpen;
    mocks.panelByTool.cursor = "cursor";
    mocks.resetByTool.cursor = false;

    await useToolPanel("screenshot").toggle("cursor", anchor);

    expect(mocks.close).not.toHaveBeenCalled();
    expect(mocks.hidePopupPanel).not.toHaveBeenCalled();
    const panel = "tool-panel-screenshot";
    expect(mocks.open).toHaveBeenCalledExactlyOnceWith(panel, {
      content: { kind: "tool", tool: "cursor", workspace: "screenshot" },
      focusContents: false,
      id: "tool:cursor",
    });
    expect(mocks.showPopupPanel).toHaveBeenCalledWith(
      expect.objectContaining({ panel }),
    );
  });

  it("does not refit when an already open panel is toggled closed", async () => {
    mocks.active = cursorPanelOpen;
    mocks.panelByTool.cursor = "cursor";
    mocks.resetByTool.cursor = true;
    const panel = useToolPanel("recording");

    await panel.toggle("cursor", anchor);

    expect(mocks.fitPreview).not.toHaveBeenCalled();
    expect(mocks.openToolPanelSpace).not.toHaveBeenCalled();
  });
});
