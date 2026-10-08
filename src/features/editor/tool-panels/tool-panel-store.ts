// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";

import {
  Background,
  BackgroundPreset,
} from "../../../components/shared/background-picker/background";
import { CameraOutput } from "../export/camera-output";
import {
  DEFAULT_CURSOR_EFFECTS,
  DEFAULT_KEYBOARD_EFFECTS,
} from "../export/recording-export-settings";
import { SceneTemplate } from "../recording/scenes/recording-scene-template";
import {
  CursorEffectSettings,
  EditorKind,
  KeyboardEffectSettings,
} from "../types";
import { createWorkspaceMirror } from "../workspace-mirror";

import { ToolPanelFrame } from "./tool-panel-frame";
import { ToolPanelPatch } from "./tool-panel-patch";
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
  ToolPanelMicrophone,
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
