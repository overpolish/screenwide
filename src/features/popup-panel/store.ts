// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";

import { EditorKind } from "../editor/types";

import type { PopupPanelIcon } from "./popup-panel-item-glyph";
import type { CaptionStyle } from "../../bindings/CaptionStyle";
import type { CaptionTemplate } from "../../bindings/CaptionTemplate";
import type { NoteTranscript } from "../editor/timeline/moments/recording-moments";

type PopupPanelMode = "menu" | "select";

export type PopupPanelItem = {
  id: string;
  label: string;
  /** A short note after the label, drawn as a badge at the trailing edge:
   * a scale beside a size, say. */
  detail?: string;
  /** The typeface the label is set in, for a list of fonts or weights that
   * shows each as it looks: a CSS family name and, if given, a weight and
   * whether italic. */
  font?: { family: string; italic?: boolean; weight?: number };
  /** A glyph from the popup panel window's registry, for items without artwork.
   */
  icon?: PopupPanelIcon;
  iconPath?: string | null;
  /** Heads the run of consecutive items that name the same section. Items
   * without one sit in an unheaded group. */
  section?: string;
  /** The key that does the same as this item, drawn as a shortcut hint at
   * the trailing edge the way a menu shows one, in the hotkey format
   * Settings stores (`Shift+]`), so each platform names its own modifiers. */
  shortcut?: string;
  /** A toggle in a single-selection list: a press flips its tick and leaves
   * the panel open, so several can be set in one visit. */
  togglesInPlace?: boolean;
};

/** The editor tools that own a panel, with room for the background, camera and
 * audio panels that follow. "annotation" is the one that belongs to what is
 * selected rather than to a tool: it dresses the arrow or counter in hand. */
export type ToolPanelKind =
  | "annotation"
  | "captions"
  | "crop"
  | "cursor"
  | "frame"
  | "keyboard"
  | "scene"
  | "selection"
  | "transcript";

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
  /** A long list found by typing: the search field's name, which heads the
   * list and takes the keys while it is open. */
  searchLabel?: string;
  /** Shown whole rather than scrolled past the list ceiling: a short menu
   * whose every entry should be in view at once. */
  showsAllItems?: boolean;
};

/** An editor tool's own controls, rendered by the workspace that owns them
 * and kept open while the user works in the editor. */
export type PopupPanelToolContent = {
  kind: "tool";
  tool: ToolPanelKind;
  workspace: EditorKind;
};

/** A moment's voice note, to play, drawn from what the editor read of it.
 * The panel fetches the sound itself. */
export type PopupPanelNoteContent = {
  artifactId: number;
  /** `#rrggbb`, the moment's kind's. */
  color: string;
  durationMs: number;
  kind: "note";
  /** Which moment it is, counting from zero in the order they were placed. */
  moment: number;
  name: string;
  /** What was said, as far as transcription has got; null before anything
   * is known of it. */
  transcript: NoteTranscript | null;
  /** Peak levels from 0 to 1. */
  waveform: number[];
};

/** The caption templates to choose from, as tiles of their moving previews.
 * `styles` are each as choosing it would set the captions. */
export type PopupPanelCaptionTemplatesContent = {
  kind: "caption-templates";
  selected: CaptionTemplate;
  styles: CaptionStyle[];
};

/** A colour grid: the palette, `savedColors`, and Custom. */
export type PopupPanelColorsContent = {
  kind: "colors";
  savedColors: string[];
  value: string;
};

export type PopupPanelContent =
  | PopupPanelCaptionTemplatesContent
  | PopupPanelColorsContent
  | PopupPanelListContent
  | PopupPanelNoteContent
  | PopupPanelToolContent;

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
  keepOnly: (panels: string[]) => void;
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

/** Each panel window's entry is stored under a key of its own, so a window
 * only ever writes the panel it changes. Every window runs in a process of
 * its own and reads the others' writes a moment late; writing the whole map
 * from such a copy put back stale entries for the panels it did not touch,
 * which emptied a tool panel, or swapped it, at a pick in a pop-up. */
const PANEL_KEY_PREFIX = "screenwide-popup-panel:";
const SELECTION_KEY = "screenwide-popup-panel-selection";
/** Where every panel was once kept together, cleared away on load. */
const OLD_PANELS_KEY = "screenwide-standalone-listbox";

const panelKey = (panel: string) => `${PANEL_KEY_PREFIX}${panel}`;

/** A stored entry, or nothing where there is none or it cannot be read. */
const parsedPanel = (raw: string | null): OpenPopupPanel | null => {
  if (!raw) return null;
  try {
    return JSON.parse(raw) as OpenPopupPanel;
  } catch {
    return null;
  }
};

/** Keeps `panel`'s entry, or takes it away for null. Storage that cannot be
 * written leaves the other windows as they were, and this one still right. */
const storePanel = (panel: string, entry: OpenPopupPanel | null) => {
  try {
    if (entry) localStorage.setItem(panelKey(panel), JSON.stringify(entry));
    else localStorage.removeItem(panelKey(panel));
  } catch {
    // Nothing to do: the next change writes it again.
  }
};

/** Every panel's stored entry, by window label. */
const storedPanels = () => {
  const active: Record<string, OpenPopupPanel | null> = {};
  try {
    localStorage.removeItem(OLD_PANELS_KEY);
    localStorage.removeItem(`${OLD_PANELS_KEY}-selection`);
    for (let index = 0; index < localStorage.length; index++) {
      const key = localStorage.key(index);
      if (key?.startsWith(PANEL_KEY_PREFIX))
        active[key.slice(PANEL_KEY_PREFIX.length)] = parsedPanel(
          localStorage.getItem(key),
        );
    }
  } catch {
    // Without storage, each window starts with nothing open.
  }
  return active;
};

export const usePopupPanelStore = create<PopupPanelStore>()((set, get) => {
  const put = (panel: string, entry: OpenPopupPanel | null) => {
    storePanel(panel, entry);
    set((state) => ({ active: { ...state.active, [panel]: entry } }));
  };
  return {
    active: storedPanels(),
    close: (panel) => {
      put(panel, null);
    },
    closeAll: () => {
      for (const panel of Object.keys(storedPanels())) storePanel(panel, null);
      set({ active: {} });
    },
    /** Drops every entry but the panels named, which are the ones still on
     * screen: the rest are left over from a run that ended. */
    keepOnly: (panels) => {
      for (const panel of Object.keys(storedPanels()))
        if (!panels.includes(panel)) storePanel(panel, null);
      set((state) => ({
        active: Object.fromEntries(
          Object.entries(state.active).filter(([panel]) =>
            panels.includes(panel),
          ),
        ),
      }));
    },
    lastSelection: null,
    open: put,
    select: ({ id, panel, pressedId, selectedIds }) => {
      const lastSelection = {
        eventId: crypto.randomUUID(),
        id,
        pressedId,
        selectedIds,
      };
      // The list keeps what it now has chosen, where it is still the one
      // the pick was made in.
      const open = get().active[panel] ?? null;
      if (open?.id === id && open.content.kind === "list")
        put(panel, { ...open, content: { ...open.content, selectedIds } });
      set({ lastSelection });
      try {
        localStorage.setItem(SELECTION_KEY, JSON.stringify(lastSelection));
      } catch {
        // The pick still reaches this window's own listeners.
      }
    },
  };
});

/** Takes in another window's change: the one panel entry it wrote, or the
 * pick it reported. */
export const synchronizePopupPanelStore = (
  event: Pick<StorageEvent, "key" | "newValue">,
) => {
  const { key } = event;
  if (key?.startsWith(PANEL_KEY_PREFIX)) {
    const panel = key.slice(PANEL_KEY_PREFIX.length);
    const entry = parsedPanel(event.newValue);
    usePopupPanelStore.setState((state) => ({
      active: { ...state.active, [panel]: entry },
    }));
  } else if (key === SELECTION_KEY && event.newValue) {
    try {
      const lastSelection = JSON.parse(event.newValue) as PopupPanelSelection;
      usePopupPanelStore.setState({ lastSelection });
    } catch {
      // Ignore malformed cross-window messages.
    }
  }
};
