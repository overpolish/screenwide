// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

import { EditorKind } from "../editor/types";

export type PopupPanelIcon = "clipboard" | "image" | "scrolling" | "trash";

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

/** The editor tools that own a panel, with room for the background,
 * keyboard, camera and audio panels that follow. */
export type ToolPanelKind = "cursor" | "frame" | "selection";

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

export type PopupPanelContent = PopupPanelListContent | PopupPanelToolContent;

/** The one panel window every pop-up button borrows. A caller that names no
 * panel means this one. */
export const SHARED_POPUP_PANEL = "standalone-listbox";

export type OpenPopupPanel = {
  /** What the panel window draws: a list of choices, or a tool's controls. */
  content: PopupPanelContent;
  focusContents: boolean;
  id: string;
  /** Names a list of choices for assistive technology. A tool panel is the
   * tool in hand and carries no chrome of its own to name. */
  label?: string;
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
  /** What each panel window is showing, keyed by its window label. The
   * editors have one each, so a tool in hand in one workspace cannot take the
   * other's panel away, and neither disturbs the shared listbox. */
  active: Record<string, OpenPopupPanel | null>;
  close: (panel: string) => void;
  /** Every panel window at once, for the app launch that must not inherit
   * whatever the last run left open. */
  closeAll: () => void;
  lastSelection: PopupPanelSelection | null;
  open: (panel: string, listbox: OpenPopupPanel) => void;
  select: (selection: PopupPanelSelectionRequest) => void;
};

/** A press in a list: which panel window reported it, which list it was, and
 * what that leaves selected. */
type PopupPanelSelectionRequest = {
  id: string;
  panel: string;
  selectedIds: string[];
  pressedId?: string;
};

/** What one panel window is showing, or nothing. */
export const activePopupPanel = (
  state: Pick<PopupPanelStore, "active">,
  panel: string,
) => state.active[panel] ?? null;

const STORE_NAME = "screenwide-standalone-listbox";
const SELECTION_STORE_NAME = `${STORE_NAME}-selection`;

export const usePopupPanelStore = create<PopupPanelStore>()(
  persist(
    (set) => ({
      active: {},
      close: (panel) => {
        set((state) => ({ active: { ...state.active, [panel]: null } }));
      },
      closeAll: () => {
        set({ active: {} });
      },
      lastSelection: null,
      open: (panel, active) => {
        set((state) => ({ active: { ...state.active, [panel]: active } }));
      },
      select: ({ id, panel, pressedId, selectedIds }) => {
        const lastSelection = {
          eventId: crypto.randomUUID(),
          id,
          pressedId,
          selectedIds,
        };
        set((state) => {
          const open = state.active[panel] ?? null;
          return {
            active:
              open?.id === id && open.content.kind === "list"
                ? {
                    ...state.active,
                    [panel]: {
                      ...open,
                      content: { ...open.content, selectedIds },
                    },
                  }
                : state.active,
            lastSelection,
          };
        });
        localStorage.setItem(
          SELECTION_STORE_NAME,
          JSON.stringify(lastSelection),
        );
      },
    }),
    {
      // The open panel is presentation state, never worth carrying across a
      // shape change: an upgrade drops whatever the last run left behind.
      migrate: () => ({ active: {} }),
      name: STORE_NAME,
      partialize: (state) => ({ active: state.active }),
      storage: createJSONStorage(() => localStorage),
      version: 2,
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
