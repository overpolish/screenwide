// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { BackgroundPreset } from "../../../components/shared/background-picker/background";
import { CaptionsPatch } from "../recording/captions/recording-captions";
import { CaptionDefaultsPatch } from "../recording/captions/use-caption-templates";
import { SceneTemplate } from "../recording/scenes/recording-scene-template";
import {
  applyKeyboardShortcutToAll,
  placeKeyboardShortcut,
  resetKeyboardShortcut,
} from "../shortcuts/keyboard-shortcut-channel";
import {
  CursorEffectSettings,
  EditorKind,
  KeyboardEffectSettings,
} from "../types";

import { annotationPanelHandlers } from "./annotation/annotation-handlers";
import { cropPanelHandlers, EditorCropTarget } from "./crop/crop-target";
import { EditorFrameTarget, framePanelHandlers } from "./frame/frame-target";
import { scenePanelHandlers } from "./scene/scene-panel-handlers";
import { EditorSelectionTarget } from "./selection/placed-selection-target";
import {
  audioPanelHandlers,
  selectionPanelHandlers,
} from "./selection/selection-panel-handlers";
import { EditorAudioSelectionTarget } from "./selection/selection-target";
import { ToolPanelHandlers } from "./tool-panel-handlers";
import { TranscriptRequest } from "./tool-panel-transcript";

/**
 * Everything the tool panels can ask for, answered against what the editor
 * already owns. Kept apart from the snapshot so `use-editor-tool-panels`
 * only wires the two halves together.
 */
export type EditorToolPanelHandlerInputs = {
  annotationColors: string[];
  audioTarget: EditorAudioSelectionTarget | null;
  backgroundPresets: BackgroundPreset[];
  cropTarget: EditorCropTarget | null;
  frameTarget: EditorFrameTarget | null;
  keyboardEffects: KeyboardEffectSettings;
  restoreShortcuts: () => void;
  saveSceneTemplates: (templates: SceneTemplate[]) => void;
  sceneTemplates: SceneTemplate[];
  selectionTarget: EditorSelectionTarget | null;
  setAnnotationColors: (colors: string[]) => void;
  setBackgroundPresets: (presets: BackgroundPreset[]) => void;
  workspace: EditorKind;
  onBakeCameraChange?: (bake: boolean) => void;
  onCaptionDefaultsChange?: (patch: CaptionDefaultsPatch) => void;
  onCaptionsChange?: (patch: CaptionsPatch) => void;
  onCursorEffectsChange?: (settings: CursorEffectSettings) => void;
  onKeyboardEffectsChange?: (settings: KeyboardEffectSettings) => void;
  onTranscriptRequest?: (request: TranscriptRequest) => void;
  resetShortcuts?: () => void;
};

/** The handlers the panels reach, each one the editor's own commit path. */
export function editorToolPanelHandlers({
  annotationColors,
  audioTarget,
  backgroundPresets,
  cropTarget,
  frameTarget,
  keyboardEffects,
  onBakeCameraChange,
  onCaptionDefaultsChange,
  onCaptionsChange,
  onCursorEffectsChange,
  onKeyboardEffectsChange,
  onTranscriptRequest,
  resetShortcuts,
  restoreShortcuts,
  saveSceneTemplates,
  sceneTemplates,
  selectionTarget,
  setAnnotationColors,
  setBackgroundPresets,
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
    onBakeCameraChange,
    onCaptionDefaultsChange,
    onCaptionsChange,
    onCursorEffectsChange,
    onKeyboardEffectsChange: (settings) => {
      onKeyboardEffectsChange?.({ ...keyboardEffects, ...settings });
    },
    // The default place is the absence of a position: the bridge is JSON,
    // which cannot carry an undefined, so the reset is its own request.
    onKeyboardPositionReset: () => {
      const {
        positionXPercent: _x,
        positionYPercent: _y,
        ...rest
      } = keyboardEffects;
      onKeyboardEffectsChange?.(rest);
    },
    onKeyboardShortcutsResetAll: resetShortcuts,
    onKeyboardShortcutsRestore: restoreShortcuts,
    onShortcutApplyToAll: () => {
      applyKeyboardShortcutToAll(workspace);
    },
    onShortcutPlacementChange: (placement) => {
      placeKeyboardShortcut(workspace, placement);
    },
    onShortcutReset: () => {
      resetKeyboardShortcut(workspace);
    },
    ...audioPanelHandlers(audioTarget),
    ...cropPanelHandlers(cropTarget),
    ...framePanelHandlers(frameTarget),
    ...selectionPanelHandlers(selectionTarget),
    ...scenePanelHandlers(workspace, {
      saveTemplates: saveSceneTemplates,
      sceneTemplates,
    }),
    onTranscriptRequest,
  };
}
