// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CaptionsPatch } from "../recording/captions/recording-captions";
import { CaptionDefaultsPatch } from "../recording/captions/use-text-templates";
import { MicrophoneTools } from "../recording/microphone/use-microphone-tools";
import { SceneTemplate } from "../recording/scenes/recording-scene-template";
import {
  CursorEffectSettings,
  EditorKind,
  KeyboardEffectSettings,
} from "../types";

import { annotationPanelHandlers } from "./annotation/annotation-handlers";
import { cropPanelHandlers, EditorCropTarget } from "./crop/crop-target";
import { BackgroundPreset } from "./frame/background/background";
import { EditorFrameTarget, framePanelHandlers } from "./frame/frame-target";
import { scenePanelHandlers } from "./scene/scene-panel-handlers";
import { EditorSelectionTarget } from "./selection/placed-selection-target";
import {
  microphonePanelHandlers,
  selectionPanelHandlers,
} from "./selection/selection-panel-handlers";
import { stripPanelHandlers } from "./strip/strip-panel-handlers";
import { ToolPanelHandlers } from "./tool-panel-handlers";
import { TranscriptRequest } from "./tool-panel-transcript";
import {
  EditorTracksInputs,
  tracksPanelHandlers,
} from "./tracks/editor-tracks";

/**
 * Everything the tool panels can ask for, answered against what the editor
 * already owns. Kept apart from the snapshot so `use-editor-tool-panels`
 * only wires the two halves together.
 */
export type EditorToolPanelHandlerInputs = {
  annotationColors: string[];
  backgroundPresets: BackgroundPreset[];
  cropTarget: EditorCropTarget | null;
  frameTarget: EditorFrameTarget | null;
  keyboardEffects: KeyboardEffectSettings;
  /** The recording's microphone's speech tools, where it has one. */
  microphone: MicrophoneTools | null;
  restoreShortcuts: () => void;
  saveSceneTemplates: (templates: SceneTemplate[]) => void;
  sceneTemplates: SceneTemplate[];
  selectionTarget: EditorSelectionTarget | null;
  setAnnotationColors: (colors: string[]) => void;
  setBackgroundPresets: (presets: BackgroundPreset[]) => void;
  /** The recorded tracks the export holds, which the Tracks island
   * switches. */
  tracks: EditorTracksInputs;
  workspace: EditorKind;
  onCaptionDefaultsChange?: (patch: CaptionDefaultsPatch) => void;
  onCaptionsChange?: (patch: CaptionsPatch) => void;
  onCursorEffectsChange?: (settings: CursorEffectSettings) => void;
  onKeyboardEffectsChange?: (settings: KeyboardEffectSettings) => void;
  onTrackVolumeChange?: (trackId: string, decibels: number) => void;
  onTranscriptRequest?: (request: TranscriptRequest) => void;
};

/** The handlers the panels reach, each one the editor's own commit path. */
export function editorToolPanelHandlers({
  annotationColors,
  backgroundPresets,
  cropTarget,
  frameTarget,
  keyboardEffects,
  microphone,
  onCaptionDefaultsChange,
  onCaptionsChange,
  onCursorEffectsChange,
  onKeyboardEffectsChange,
  onTrackVolumeChange,
  onTranscriptRequest,
  restoreShortcuts,
  saveSceneTemplates,
  sceneTemplates,
  selectionTarget,
  setAnnotationColors,
  setBackgroundPresets,
  tracks,
  workspace,
}: EditorToolPanelHandlerInputs): ToolPanelHandlers {
  return {
    ...annotationPanelHandlers(workspace, annotationColors, (colors) => {
      setAnnotationColors(colors);
    }),
    onBackgroundPresetRemove: (id) => {
      setBackgroundPresets(
        backgroundPresets.filter((preset) => preset.id !== id),
      );
    },
    onBackgroundPresetSave: (preset) => {
      setBackgroundPresets([
        ...backgroundPresets.filter(
          (saved) => saved.id !== preset.id && saved.name !== preset.name,
        ),
        preset,
      ]);
    },
    onCaptionDefaultsChange,
    onCaptionsChange,
    onCursorEffectsChange,
    onKeyboardEffectsChange: (settings) => {
      onKeyboardEffectsChange?.({ ...keyboardEffects, ...settings });
    },
    onKeyboardShortcutsRestore: restoreShortcuts,
    ...microphonePanelHandlers(microphone),
    onTrackVolumeChange,
    ...cropPanelHandlers(cropTarget),
    ...framePanelHandlers(frameTarget),
    ...selectionPanelHandlers(selectionTarget),
    ...scenePanelHandlers(workspace, {
      saveTemplates: saveSceneTemplates,
      sceneTemplates,
    }),
    ...stripPanelHandlers(workspace),
    ...tracksPanelHandlers(tracks),
    onTranscriptRequest,
  };
}
