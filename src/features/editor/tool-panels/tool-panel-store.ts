// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

import {
  Background,
  BackgroundPreset,
} from "../../../components/shared/background-picker/background";
import { AnnotationStyle } from "../annotations";
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

import { ToolPanelFrame } from "./tool-panel-frame";
import {
  ToolPanelAnnotation,
  ToolPanelSelection,
} from "./tool-panel-selection";

/**
 * What the frame panel shows: the output canvas the workspace renders into,
 * and the source size a reset puts it back to.
 */
export type { ToolPanelFrame } from "./tool-panel-frame";

/** What the selection panel places. */
export type {
  ToolPanelAudioSelection,
  ToolPanelLayerSelection,
  ToolPanelShortcutSelection,
} from "./tool-panel-selection";

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
  /** The annotation the preview has in hand, or null while it has none. Unlike
   * every other panel, the arrow panel follows this rather than a tool. */
  annotation: ToolPanelAnnotation | null;
  /** Colours of your own the annotation tools were given, newest last. */
  annotationColors: string[];
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
  /** A save runs or a sheet stands over the editor: the panels stay up, and
   * every control in them is disabled. */
  isLocked: boolean;
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
  /** Turn the chosen counter's tail, in radians clockwise from east. Where it
   * points is the annotation's own property rather than part of its dress, and
   * it becomes the next counter's aim. */
  annotationAngle?: number;
  /** Draw the chosen annotation in and out over its clip, or leave it standing.
   * This is the annotation's own property rather than part of its dress, so it
   * travels beside the style rather than inside it. It also becomes the next
   * arrow's default. */
  annotationAnimated?: boolean;
  /** Dress the chosen annotation, a field at a time so a panel never has to
   * send the whole style back. Each one also becomes the next arrow's default.
   */
  annotationStyle?: Partial<AnnotationStyle>;
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
  /** Round the output canvas corners by this share of its shorter side. */
  frameRadius?: number;
  /** Size the output canvas, leaving what is in it where it sits. */
  frameSize?: { height?: number; width?: number };
  /** The shortcut settings every shortcut is drawn with, changed a field at a
   * time so a panel never has to send the whole group back. */
  keyboardEffects?: Partial<KeyboardEffectSettings>;
  /** Put the selected layer's content in the middle of its padded frame. */
  recenterSelection?: true;
  /** Forget a colour of your own, by the colour itself. */
  removeAnnotationColor?: string;
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
  /** Turn the chosen annotation round, so its head points the other way. */
  reverseAnnotation?: true;
  /** Keep a colour of your own, so it is on offer next time. Sent once a
   * colour is settled on rather than on every step of a drag. */
  saveAnnotationColor?: string;
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
  /** Lay the chosen redaction's blocks out again from a fresh seed. */
  shuffleAnnotation?: true;
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
  annotation: null,
  annotationColors: [],
  background: { color: "#171717", kind: "solid" },
  backgroundPresets: [],
  canRestoreShortcuts: false,
  crop: null,
  cursorEffects: DEFAULT_CURSOR_EFFECTS,
  frame: null,
  hasCursorData: false,
  hasKeyboardData: false,
  isLocked: false,
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
