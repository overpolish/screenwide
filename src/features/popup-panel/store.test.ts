// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

// Hoisted above the store import: the module persists itself on creation, so
// the stand-in has to be in place before it is loaded.
const store = vi.hoisted(() => {
  const entries = new Map<string, string>();
  vi.stubGlobal("localStorage", {
    getItem: (key: string) => entries.get(key) ?? null,
    removeItem: (key: string) => {
      entries.delete(key);
    },
    setItem: (key: string, value: string) => {
      entries.set(key, value);
    },
  });
  return entries;
});

import {
  activePopupPanel,
  OpenPopupPanel,
  SHARED_POPUP_PANEL,
  usePopupPanelStore,
} from "./store";

const RECORDING_PANEL = "tool-panel-recording";
const SCREENSHOT_PANEL = "tool-panel-screenshot";

const toolPanel = (
  tool: "cursor" | "selection",
  workspace: "recording" | "screenshot",
): OpenPopupPanel => ({
  content: { kind: "tool", tool, workspace },
  focusContents: false,
  id: `tool:${tool}`,
});

const list = (id: string, selectedIds: string[]): OpenPopupPanel => ({
  content: {
    items: [
      { id: "one", label: "One" },
      { id: "two", label: "Two" },
    ],
    kind: "list",
    mode: "select",
    selectedIds,
    selectionMode: "single",
  },
  focusContents: false,
  id,
});

const open = (panel: string, popup: OpenPopupPanel) => {
  usePopupPanelStore.getState().open(panel, popup);
};

beforeEach(() => {
  usePopupPanelStore.getState().closeAll();
});

afterEach(() => {
  store.clear();
});

describe("usePopupPanelStore", () => {
  it("holds one open panel per window", () => {
    open(RECORDING_PANEL, toolPanel("cursor", "recording"));
    open(SCREENSHOT_PANEL, toolPanel("selection", "screenshot"));
    open(SHARED_POPUP_PANEL, list("camera", []));

    const state = usePopupPanelStore.getState();
    expect(activePopupPanel(state, RECORDING_PANEL)?.id).toBe("tool:cursor");
    expect(activePopupPanel(state, SCREENSHOT_PANEL)?.id).toBe(
      "tool:selection",
    );
    expect(activePopupPanel(state, SHARED_POPUP_PANEL)?.id).toBe("camera");
  });

  it("opening a panel in one workspace leaves the other's alone", () => {
    open(RECORDING_PANEL, toolPanel("cursor", "recording"));
    const before = activePopupPanel(
      usePopupPanelStore.getState(),
      RECORDING_PANEL,
    );

    open(SCREENSHOT_PANEL, toolPanel("cursor", "screenshot"));

    expect(
      activePopupPanel(usePopupPanelStore.getState(), RECORDING_PANEL),
    ).toBe(before);
    expect(
      activePopupPanel(usePopupPanelStore.getState(), SCREENSHOT_PANEL)
        ?.content,
    ).toEqual({ kind: "tool", tool: "cursor", workspace: "screenshot" });
  });

  it("closes only the panel it is given", () => {
    open(RECORDING_PANEL, toolPanel("cursor", "recording"));
    open(SCREENSHOT_PANEL, toolPanel("cursor", "screenshot"));

    usePopupPanelStore.getState().close(RECORDING_PANEL);

    const state = usePopupPanelStore.getState();
    expect(activePopupPanel(state, RECORDING_PANEL)).toBeNull();
    expect(activePopupPanel(state, SCREENSHOT_PANEL)).not.toBeNull();
  });

  it("keeps a selection in the panel that reported it", () => {
    open(SHARED_POPUP_PANEL, list("camera", []));
    open(RECORDING_PANEL, toolPanel("cursor", "recording"));

    usePopupPanelStore.getState().select({
      id: "camera",
      panel: SHARED_POPUP_PANEL,
      pressedId: "two",
      selectedIds: ["two"],
    });

    const state = usePopupPanelStore.getState();
    const shared = activePopupPanel(state, SHARED_POPUP_PANEL);
    expect(
      shared?.content.kind === "list" && shared.content.selectedIds,
    ).toEqual(["two"]);
    expect(state.lastSelection).toMatchObject({
      id: "camera",
      pressedId: "two",
      selectedIds: ["two"],
    });
    expect(activePopupPanel(state, RECORDING_PANEL)?.id).toBe("tool:cursor");
  });

  it("reports a selection made in a panel showing something else", () => {
    open(SHARED_POPUP_PANEL, list("camera", ["one"]));

    usePopupPanelStore.getState().select({
      id: "microphone",
      panel: SHARED_POPUP_PANEL,
      selectedIds: ["two"],
    });

    const shared = activePopupPanel(
      usePopupPanelStore.getState(),
      SHARED_POPUP_PANEL,
    );
    expect(
      shared?.content.kind === "list" && shared.content.selectedIds,
    ).toEqual(["one"]);
    expect(usePopupPanelStore.getState().lastSelection?.id).toBe("microphone");
  });

  it("clears every panel at once on launch", () => {
    open(RECORDING_PANEL, toolPanel("cursor", "recording"));
    open(SHARED_POPUP_PANEL, list("camera", []));

    usePopupPanelStore.getState().closeAll();

    expect(usePopupPanelStore.getState().active).toEqual({});
  });
});
