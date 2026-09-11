// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

import { DEFAULT_CURSOR_EFFECTS } from "../recording-export-settings";
import { SelectionPlacementPatch } from "../selection-placement";
import { CursorEffectSettings, EditorKind } from "../types";

/**
 * What the selection panel shows: the selected layer, its size and position in
 * output pixels, and the source it was captured at, which is what a reset
 * puts it back to.
 */
export type ToolPanelSelection = {
  height: number;
  /** Which of the workspace's layers this is, for wording that fits it. */
  kind: "camera" | "layer" | "primary";
  label: string;
  sourceHeight: number;
  sourceWidth: number;
  width: number;
  x: number;
  y: number;
};

/**
 * What the frame panel shows: the output canvas the workspace renders into,
 * and the source size a reset puts it back to.
 */
export type ToolPanelFrame = {
  height: number;
  sourceHeight: number;
  sourceWidth: number;
  width: number;
};

/**
 * Everything the tool panels show that the editor window owns.
 *
 * One snapshot per workspace, published by the editor and read by the panel
 * window. Later panels add their own fields here: the keyboard, camera, audio
 * and background settings each arrive as more keys, not another mirror.
 */
export type ToolPanelSnapshot = {
  cursorEffects: CursorEffectSettings;
  /** Null until the workspace has an output canvas to size. */
  frame: ToolPanelFrame | null;
  hasCursorData: boolean;
  isSaving: boolean;
  /** Null while the workspace has nothing selected to place. */
  selection: ToolPanelSelection | null;
  /** Latest panel request committed with this snapshot. */
  acknowledgedSeq?: number;
};

/** The settings a panel may ask the editor to change. */
export type ToolPanelPatch = Partial<
  Pick<ToolPanelSnapshot, "cursorEffects">
> & {
  /** Size the output canvas, leaving what is in it where it sits. */
  frameSize?: { height?: number; width?: number };
  /** Put the canvas back to the source size, refitting what is in it. */
  resetFrame?: true;
  /** Put the selection's size and position back to its source framing. */
  resetSelection?: true;
  selectionOutput?: SelectionPlacementPatch;
};

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
  frame: null,
  hasCursorData: false,
  isSaving: false,
  selection: null,
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
