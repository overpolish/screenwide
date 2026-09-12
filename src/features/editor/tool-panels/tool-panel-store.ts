// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

import {
  Background,
  BackgroundPreset,
} from "../../../components/shared/background-picker/background";
import {
  DEFAULT_CURSOR_EFFECTS,
  DEFAULT_KEYBOARD_EFFECTS,
} from "../recording-export-settings";
import { SelectionPlacementPatch } from "../selection-placement";
import {
  CursorEffectSettings,
  EditorKind,
  KeyboardEffectSettings,
} from "../types";

/**
 * What the selection panel shows for a placed layer: the selected layer, its
 * size and position in output pixels, and the source it was captured at, which
 * is what a reset puts it back to.
 */
export type ToolPanelLayerSelection = {
  /** Whether this layer casts a shadow onto the canvas behind it. */
  dropShadow: boolean;
  height: number;
  /** How far the padded frame runs past the layer's own picture, in output
   * pixels, on each side. */
  inset: number;
  /** As far as the padding may be taken: the layer's shorter side. */
  insetMaximum: number;
  /** Which of the workspace's layers this is, for wording that fits it. */
  kind: "camera" | "layer" | "primary";
  label: string;
  /** How rounded the layer's corners are, as a share of its shorter side,
   * 0 to 50. The same number the corner drag in the preview sets. */
  radius: number;
  sourceHeight: number;
  sourceWidth: number;
  width: number;
  x: number;
  y: number;
  /** Camera only: whether baking it in is on the table at all. Baking draws
   * the camera into the screen's picture, so it needs both tracks kept. */
  canBake?: boolean;
  /** Camera only: whether it is drawn into the screen's picture rather than
   * carried as a track of its own. */
  isBaked?: boolean;
};

/**
 * What the selection panel shows for the keyboard shortcut on screen.
 *
 * A shortcut is drawn rather than placed: it has no source pixels, no corners
 * and no pad, so it is sized as a share of its natural size and positioned by
 * its centre, both in percent - the very numbers the drag on it sets.
 */
export type ToolPanelShortcutSelection = {
  kind: "shortcut";
  label: string;
  /** As big as this recording's widest shortcut may be drawn. */
  maximumSizePercent: number;
  minimumSizePercent: number;
  /** The shortcut's centre, as a share of the output canvas. */
  positionXPercent: number;
  positionYPercent: number;
  sizePercent: number;
};

/**
 * What the selection panel shows for a recorded audio track.
 *
 * An audio track is neither placed nor drawn: it is heard, so the only thing
 * there is to set for it is how loud it is played back, in decibels against
 * the level it was recorded at.
 */
export type ToolPanelAudioSelection = {
  /** How much the track is lifted or lowered, 0 being the recorded level. */
  decibels: number;
  kind: "audio";
  /** The track's own name: "Microphone", "System audio", or "Audio" where the
   * recording did not say. */
  label: string;
};

type ToolPanelSelection =
  | ToolPanelAudioSelection
  | ToolPanelLayerSelection
  | ToolPanelShortcutSelection;

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
 * What the crop panel shows: the visible rectangle of the source, in source
 * pixels, and the whole source it is cut from - which is what a reset puts it
 * back to.
 */
export type ToolPanelCrop = {
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
  /** What the workspace's canvas is filled with behind its layers. */
  background: Background;
  /** The backgrounds saved from the picker, in the order they were saved. */
  backgroundPresets: BackgroundPreset[];
  /** Whether any shortcut deleted from the timeline can be brought back. */
  canRestoreShortcuts: boolean;
  /** Null until the workspace has a source to cut a crop out of. */
  crop: ToolPanelCrop | null;
  cursorEffects: CursorEffectSettings;
  /** Null until the workspace has an output canvas to size. */
  frame: ToolPanelFrame | null;
  hasCursorData: boolean;
  hasKeyboardData: boolean;
  isSaving: boolean;
  keyboardEffects: KeyboardEffectSettings;
  /** As big as a shortcut may be drawn in this recording's canvas, in percent:
   * the point past which the widest shortcut would run off the edge. */
  keyboardMaximum: number;
  /** Null while the workspace has nothing selected to place. */
  selection: ToolPanelSelection | null;
  /** Latest panel request committed with this snapshot. */
  acknowledgedSeq?: number;
};

/** The settings a panel may ask the editor to change. */
export type ToolPanelPatch = Partial<
  Pick<ToolPanelSnapshot, "background" | "cursorEffects">
> & {
  /** Put every shortcut back where the recording drew it. */
  applyShortcutToAll?: true;
  /** Play the selected audio track this much louder or quieter than it was
   * recorded, in decibels. */
  audioVolume?: number;
  /** Draw the camera into the screen's picture, or carry it as a track of its
   * own. */
  bakeCamera?: boolean;
  /** Cut a crop of this size, in source pixels, keeping it where it sits. */
  cropSize?: { height?: number; width?: number };
  /** Size the output canvas, leaving what is in it where it sits. */
  frameSize?: { height?: number; width?: number };
  /** The shortcut settings every shortcut is drawn with, changed a field at a
   * time so a panel never has to send the whole group back. */
  keyboardEffects?: Partial<KeyboardEffectSettings>;
  /** Put the selected layer's content in the middle of its padded frame. */
  recenterSelection?: true;
  /** Forget a saved background, by its id. */
  removePreset?: string;
  /** Put every shortcut's own placement away, the global one left as it is. */
  resetAllShortcuts?: true;
  /** Show the whole source again, the committed crop taken away. */
  resetCrop?: true;
  /** Put the canvas back to the source size, refitting what is in it. */
  resetFrame?: true;
  /** Put the shortcuts back where the recording draws them by default. */
  resetKeyboardPosition?: true;
  /** Put the selection's size and position back to its source framing. */
  resetSelection?: true;
  /** Put the selected shortcut back where the recording drew it. */
  resetShortcut?: true;
  /** Bring back every shortcut deleted from the timeline. */
  restoreShortcuts?: true;
  /** Keep the background being shown under a name. */
  savePreset?: BackgroundPreset;
  /** Cast the selected layer's shadow onto the canvas, or take it away. */
  selectionDropShadow?: boolean;
  /** Pad the selected layer by this many output pixels on each side. */
  selectionInset?: number;
  selectionOutput?: SelectionPlacementPatch;
  /** Round the selected layer's corners by this share of its shorter side. */
  selectionRadius?: number;
  /** Draw the selected shortcut at this size and centre, all in percent. */
  shortcutPlacement?: {
    positionXPercent?: number;
    positionYPercent?: number;
    sizePercent?: number;
  };
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
  background: { color: "#171717", kind: "solid" },
  backgroundPresets: [],
  canRestoreShortcuts: false,
  crop: null,
  cursorEffects: DEFAULT_CURSOR_EFFECTS,
  frame: null,
  hasCursorData: false,
  hasKeyboardData: false,
  isSaving: false,
  keyboardEffects: DEFAULT_KEYBOARD_EFFECTS,
  keyboardMaximum: 500,
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
