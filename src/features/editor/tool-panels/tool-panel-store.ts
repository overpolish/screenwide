// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";

import {
  Background,
  BackgroundPreset,
} from "../../../components/shared/background-picker/background";
import { AnnotationStyle } from "../annotations/annotations";
import { CameraOutput } from "../export/camera-output";
import {
  DEFAULT_CURSOR_EFFECTS,
  DEFAULT_KEYBOARD_EFFECTS,
} from "../export/recording-export-settings";
import { SceneTemplate } from "../recording/scenes/recording-scene-template";
import { SceneVariant } from "../recording/scenes/recording-scene-variant";
import {
  RecordingScenePreset,
  SceneFraming,
} from "../recording/scenes/recording-scenes";
import {
  CursorEffectSettings,
  EditorKind,
  KeyboardEffectSettings,
} from "../types";
import { createWorkspaceMirror } from "../workspace-mirror";

import { SelectionPlacementPatch } from "./selection-placement";
import { ToolPanelFrame } from "./tool-panel-frame";
import { ToolPanelScene } from "./tool-panel-scene";
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
  /** How many annotations the preview has chosen. With more than one there is
   * no `annotation` to dress, and the panel offers only what acts on them all.
   */
  annotationCount: number;
  /** What the workspace's canvas is filled with behind its layers. */
  background: Background;
  /** The backgrounds saved from the picker, in the order they were saved. */
  backgroundPresets: BackgroundPreset[];
  /** One video or separate files; null where there is no choice to make. */
  cameraOutput: CameraOutput | null;
  /** Whether the picture being edited has any stroke to clear. */
  canClearDrawings: boolean;
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
  /** Null where the recording has no screen and camera to arrange. */
  scene: ToolPanelScene | null;
  /** The scene layouts kept for every recording, oldest first. */
  sceneTemplates: SceneTemplate[];
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
  /** Draw the camera into the screen's video, or save it as a file of its
   * own. */
  bakeCamera?: boolean;
  /** Lay the scene under the playhead out as this template. */
  chooseSceneTemplate?: SceneTemplate;
  /** Take every stroke off the picture being edited. */
  clearDrawings?: true;
  /** Cut a crop of this size, in source pixels, keeping it where it sits. */
  cropSize?: { height?: number; width?: number };
  /** Make the scene under the playhead custom, its panes kept where it puts
   * them now, or add a custom scene there where there is none. */
  customizeScene?: true;
  /** Delete every chosen annotation, in one edit. */
  deleteAnnotations?: true;
  /** Delete the scene under the playhead. */
  deleteScene?: true;
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
  /** Forget a scene template, by its id. */
  removeSceneTemplate?: string;
  /** Put every shortcut's own placement away, the global one left as it is. */
  resetAllShortcuts?: true;
  /** Show the whole source again, the committed crop taken away. */
  resetCrop?: true;
  /** Put the canvas back to the source size, refitting what is in it. */
  resetFrame?: true;
  /** Put the shortcuts back where the recording draws them by default. */
  resetKeyboardPosition?: true;
  /** Show the whole screen and camera again in the scene under the playhead. */
  resetSceneFraming?: true;
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
  /** Keep the custom scene under the playhead as a template. */
  saveSceneTemplate?: true;
  /** Zoom the scene under the playhead into its selected pane, or move the
   * part of the pane it shows, a field at a time. */
  sceneFraming?: Partial<SceneFraming>;
  /** Give the scene under the playhead this preset, or add a scene there
   * with it where there is none. */
  scenePreset?: RecordingScenePreset;
  /** Round the selected pane's corners in the scene under the playhead by
   * this share of its shorter side, in percent. */
  sceneRadius?: number;
  /** Change the options of the preset under the playhead, a field at a time. */
  sceneVariant?: SceneVariant;
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
  /** Lay the chosen redaction's blocks out again, or draw the chosen
   * hand-drawn highlight's stroke again, from a fresh seed. */
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
  annotationCount: 0,
  background: { color: "#171717", kind: "solid" },
  backgroundPresets: [],
  cameraOutput: null,
  canClearDrawings: false,
  canRestoreShortcuts: false,
  crop: null,
  cursorEffects: DEFAULT_CURSOR_EFFECTS,
  frame: null,
  hasCursorData: false,
  hasKeyboardData: false,
  isLocked: false,
  keyboardEffects: DEFAULT_KEYBOARD_EFFECTS,
  keyboardMaximum: 500,
  scene: null,
  sceneTemplates: [],
  selection: null,
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

const mirror = createWorkspaceMirror<ToolPanelSnapshot>(STORE_NAME);

export const useToolPanelStore = mirror.useMirror;

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
  mirror.synchronize(event);
  if (event.key === REQUEST_STORE_NAME && event.newValue) {
    try {
      useToolPanelRequestStore.setState({
        lastRequest: JSON.parse(event.newValue) as ToolPanelMessage,
      });
    } catch {
      // Ignore malformed cross-window messages.
    }
  }
};
