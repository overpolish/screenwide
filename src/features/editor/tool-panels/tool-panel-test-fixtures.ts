// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { afterEach, beforeEach, vi } from "vitest";

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
  fitDuringResize: vi.fn(async (_gutter: number, resize: () => Promise<void>) =>
    resize(),
  ),
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
    fitDuringResize: mocks.fitDuringResize,
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

const usePanel = (workspace: "recording" | "screenshot") =>
  useToolPanel(workspace);

export {
  anchor,
  cursorPanelOpen,
  mocks,
  setViewportWidth,
  useCursorPanel,
  usePanel,
};
