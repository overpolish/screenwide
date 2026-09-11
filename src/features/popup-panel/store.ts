// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

import { EditorKind } from "../editor/types";

export type PopupPanelIcon = "clipboard" | "image" | "scrolling";

type PopupPanelMode = "menu" | "select";

export type PopupPanelItem = {
  id: string;
  label: string;
  /** A short note after the label, drawn as a badge at the trailing edge:
   * a scale beside a size, say. */
  detail?: string;
  /** A glyph from the popup panel window's registry, for items without artwork. */
  icon?: PopupPanelIcon;
  iconPath?: string | null;
  /** Heads the run of consecutive items that name the same section. Items
   * without one sit in an unheaded group. */
  section?: string;
  /** A toggle in a single-selection list: a press flips its tick and leaves
   * the panel open, so several can be set in one visit. */
  togglesInPlace?: boolean;
};

/** The editor tools that own a panel. One for now, with room for the
 * background, keyboard, camera and audio panels that follow. */
export type ToolPanelKind = "cursor";

/** A list of choices: the panel as it has always been. */
export type PopupPanelListContent = {
  items: PopupPanelItem[];
  kind: "list";
  /** `select` shows a check gutter and keeps the pick; `menu` runs an action
   * and dismisses, so nothing is ever drawn as selected. */
  mode: PopupPanelMode;
  selectedIds: string[];
  selectionMode: "multiple" | "single";
  exclusiveId?: string;
};

/** An editor tool's own controls, rendered by the workspace that owns them
 * and kept open while the user works in the editor. */
export type PopupPanelToolContent = {
  kind: "tool";
  tool: ToolPanelKind;
  workspace: EditorKind;
};

type PopupPanelContent = PopupPanelListContent | PopupPanelToolContent;

type OpenPopupPanel = {
  /** What the panel window draws: a list of choices, or a tool's controls. */
  content: PopupPanelContent;
  focusContents: boolean;
  id: string;
  label: string;
};

type PopupPanelSelection = {
  eventId: string;
  id: string;
  selectedIds: string[];
  /** The item the press landed on. In a single-selection list this is the
   * choice; a toggle reports itself here while `selectedIds` carries the
   * whole tick list it belongs to. */
  pressedId?: string;
};

type PopupPanelStore = {
  active: OpenPopupPanel | null;
  close: () => void;
  lastSelection: PopupPanelSelection | null;
  open: (listbox: OpenPopupPanel) => void;
  select: (id: string, selectedIds: string[], pressedId?: string) => void;
};

const STORE_NAME = "screenwide-standalone-listbox";
const SELECTION_STORE_NAME = `${STORE_NAME}-selection`;

export const usePopupPanelStore = create<PopupPanelStore>()(
  persist(
    (set) => ({
      active: null,
      close: () => {
        set({ active: null });
      },
      lastSelection: null,
      open: (active) => {
        set({ active });
      },
      select: (id, selectedIds, pressedId) => {
        const lastSelection = {
          eventId: crypto.randomUUID(),
          id,
          pressedId,
          selectedIds,
        };
        set((state) => ({
          active:
            state.active?.id === id && state.active.content.kind === "list"
              ? {
                  ...state.active,
                  content: { ...state.active.content, selectedIds },
                }
              : state.active,
          lastSelection,
        }));
        localStorage.setItem(
          SELECTION_STORE_NAME,
          JSON.stringify(lastSelection),
        );
      },
    }),
    {
      // The open panel is presentation state, never worth carrying across a
      // shape change: an upgrade drops whatever the last run left behind.
      migrate: () => ({ active: null }),
      name: STORE_NAME,
      partialize: (state) => ({ active: state.active }),
      storage: createJSONStorage(() => localStorage),
      version: 1,
    },
  ),
);

export const synchronizePopupPanelStore = (event: StorageEvent) => {
  if (event.key === STORE_NAME) {
    void usePopupPanelStore.persist.rehydrate();
  } else if (event.key === SELECTION_STORE_NAME && event.newValue) {
    try {
      const lastSelection = JSON.parse(event.newValue) as PopupPanelSelection;
      usePopupPanelStore.setState({ lastSelection });
    } catch {
      // Ignore malformed cross-window messages.
    }
  }
};
