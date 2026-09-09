// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

export type PopupPanelIcon = "clipboard" | "image" | "scrolling";

type PopupPanelMode = "menu" | "select";

export type PopupPanelItem = {
  id: string;
  label: string;
  /** A glyph from the popup panel window's registry, for items without artwork. */
  icon?: PopupPanelIcon;
  iconPath?: string | null;
  /** Heads the run of consecutive items that name the same section. Items
   * without one sit in an unheaded group. */
  section?: string;
};

type OpenPopupPanel = {
  focusContents: boolean;
  id: string;
  items: PopupPanelItem[];
  label: string;
  /** `select` shows a check gutter and keeps the pick; `menu` runs an action
   * and dismisses, so nothing is ever drawn as selected. */
  mode: PopupPanelMode;
  selectedIds: string[];
  selectionMode: "multiple" | "single";
  exclusiveId?: string;
};

type PopupPanelSelection = {
  eventId: string;
  id: string;
  selectedIds: string[];
};

type PopupPanelStore = {
  active: OpenPopupPanel | null;
  close: () => void;
  lastSelection: PopupPanelSelection | null;
  open: (listbox: OpenPopupPanel) => void;
  select: (id: string, selectedIds: string[]) => void;
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
      select: (id, selectedIds) => {
        const lastSelection = {
          eventId: crypto.randomUUID(),
          id,
          selectedIds,
        };
        set((state) => ({
          active:
            state.active?.id === id
              ? { ...state.active, selectedIds }
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
      name: STORE_NAME,
      partialize: (state) => ({ active: state.active }),
      storage: createJSONStorage(() => localStorage),
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
