// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

import { DEFAULT_CURSOR_EFFECTS } from "../recording-export-settings";
import { CursorEffectSettings, EditorKind } from "../types";

/**
 * Everything the tool panels show that the editor window owns.
 *
 * One snapshot per workspace, published by the editor and read by the panel
 * window. Later panels add their own fields here: the keyboard, camera, audio
 * and background settings each arrive as more keys, not another mirror.
 */
export type ToolPanelSnapshot = {
  cursorEffects: CursorEffectSettings;
  hasCursorData: boolean;
  isSaving: boolean;
  /** Latest panel request committed with this snapshot. */
  acknowledgedSeq?: number;
};

/** The settings a panel may ask the editor to change. */
export type ToolPanelPatch = Partial<Pick<ToolPanelSnapshot, "cursorEffects">>;

export type ToolPanelRequest =
  | { type: "patch"; values: ToolPanelPatch }
  | { event: KeyboardEventInit; type: "shortcut" };

export type ToolPanelMessage = {
  request: ToolPanelRequest;
  /** Rising, so the editor can tell a repeat delivery from a new request. */
  seq: number;
  workspace: EditorKind;
};

export const DEFAULT_TOOL_PANEL_SNAPSHOT: ToolPanelSnapshot = {
  cursorEffects: DEFAULT_CURSOR_EFFECTS,
  hasCursorData: false,
  isSaving: false,
};

type ToolPanelStore = {
  publish: (workspace: EditorKind, snapshot: ToolPanelSnapshot) => void;
  snapshots: Partial<Record<EditorKind, ToolPanelSnapshot>>;
};

const STORE_NAME = "screenwide-tool-panels";
const REQUEST_STORE_NAME = `${STORE_NAME}-request`;

// Wall-clock seeded so a reloaded panel window cannot restart below the
// sequence the editor has already applied.
let lastSeq = 0;
const nextSeq = () => {
  lastSeq = Math.max(lastSeq + 1, Date.now());
  return lastSeq;
};

/** Settings groups travel as objects, so a snapshot key is compared by value
 * rather than by identity. */
const isSameValue = (a: unknown, b: unknown) =>
  a === b ||
  (typeof a === "object" &&
    a !== null &&
    typeof b === "object" &&
    b !== null &&
    JSON.stringify(a) === JSON.stringify(b));

const isSameSnapshot = (
  a: ToolPanelSnapshot | undefined,
  b: ToolPanelSnapshot,
) =>
  a !== undefined &&
  (Object.keys(b) as (keyof ToolPanelSnapshot)[]).every((key) =>
    isSameValue(a[key], b[key]),
  );

export const useToolPanelStore = create<ToolPanelStore>()(
  persist(
    (set) => ({
      publish: (workspace, snapshot) => {
        set((state) =>
          isSameSnapshot(state.snapshots[workspace], snapshot)
            ? state
            : { snapshots: { ...state.snapshots, [workspace]: snapshot } },
        );
      },
      snapshots: {},
    }),
    {
      name: STORE_NAME,
      partialize: (state) => ({ snapshots: state.snapshots }),
      storage: createJSONStorage(() => localStorage),
    },
  ),
);

/**
 * The latest ask from a panel, held apart from the settings mirror so sending
 * one never writes the mirror back: only the editor publishes settings, and
 * only a panel asks for changes.
 */
export const useToolPanelRequestStore = create<{
  lastRequest: ToolPanelMessage | null;
}>()(() => ({ lastRequest: null }));

/** Asks the editor that owns `workspace` to make a change. */
export const sendToolPanelRequest = (
  workspace: EditorKind,
  request: ToolPanelRequest,
) => {
  const message: ToolPanelMessage = { request, seq: nextSeq(), workspace };
  localStorage.setItem(REQUEST_STORE_NAME, JSON.stringify(message));
  return message.seq;
};

/** Carries the editor's settings out, and a panel's asks back. */
export const synchronizeToolPanelStore = (event: StorageEvent) => {
  if (event.key === STORE_NAME) {
    void useToolPanelStore.persist.rehydrate();
  } else if (event.key === REQUEST_STORE_NAME && event.newValue) {
    try {
      useToolPanelRequestStore.setState({
        lastRequest: JSON.parse(event.newValue) as ToolPanelMessage,
      });
    } catch {
      // Ignore malformed cross-window messages.
    }
  }
};
