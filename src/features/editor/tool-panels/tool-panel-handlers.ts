// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Background,
  BackgroundPreset,
} from "../../../components/shared/background-picker/background";
import { ImagePlayChange } from "../annotations/annotation-channel";
import { AnnotationStyle, ImageArt } from "../annotations/annotations";
import { SceneTemplate } from "../recording/scenes/recording-scene-template";
import { SceneVariant } from "../recording/scenes/recording-scene-variant";
import {
  RecordingScenePreset,
  SceneFraming,
} from "../recording/scenes/recording-scenes";
import { CursorEffectSettings, KeyboardEffectSettings } from "../types";

import { SelectionPlacementPatch } from "./selection-placement";
import { ToolPanelPatch } from "./tool-panel-patch";

/**
 * What a tool panel can ask for. Each one is an editor handler that already
 * exists, so a control in a panel and the same control in the editor take the
 * identical path through the workspace's state.
 */
export type ToolPanelHandlers = {
  /** Draw the chosen annotation in and out over its clip, or leave it standing.
   */
  onAnnotationAngleChange?: (angle: number) => void;
  onAnnotationAnimatedChange?: (animated: boolean) => void;
  /** Forget a colour of your own. */
  onAnnotationColorRemove?: (color: string) => void;
  /** Keep a colour of your own, so it is on offer next time. */
  onAnnotationColorSave?: (color: string) => void;
  /** Give the chosen image, or the next one, another picture. */
  onAnnotationImageChange?: (art: ImageArt) => void;
  /** Change how the chosen moving image plays. */
  onAnnotationImagePlayChange?: (play: ImagePlayChange) => void;
  /** Sway the chosen image over its clip, or stand it still. */
  onAnnotationImageSwayChange?: (sway: boolean) => void;
  /** Turn the chosen annotation round, through the same commit path the drag on
   * the picture uses. */
  onAnnotationReverse?: () => void;
  /** Lay the chosen redaction's blocks out again, draw the chosen hand-drawn
   * highlight's stroke again, or sway the chosen image another way, from a
   * fresh seed. */
  onAnnotationShuffle?: () => void;
  /** Dress the chosen annotation, a field at a time, through the same commit
   * path the drag on the picture uses. */
  onAnnotationStyleChange?: (style: Partial<AnnotationStyle>) => void;
  /** Delete every chosen annotation, in one edit. */
  onAnnotationsDelete?: () => void;
  /** Play the selected audio track this much louder or quieter than it was
   * recorded, in decibels. */
  onAudioVolumeChange?: (decibels: number) => void;
  /** Fill the workspace's canvas with this background. */
  onBackgroundChange?: (background: Background) => void;
  onBackgroundPresetRemove?: (id: string) => void;
  onBackgroundPresetSave?: (preset: BackgroundPreset) => void;
  /** Draw the camera into the screen's picture, or carry it as a track of its
   * own. The placement of each is carried across the change. */
  onBakeCameraChange?: (bake: boolean) => void;
  /** Show the whole source again, the committed crop taken away. */
  onCropReset?: () => void;
  /** Cut a crop of the size a field asked for, in source pixels. */
  onCropSizeChange?: (size: { height?: number; width?: number }) => void;
  onCursorEffectsChange?: (settings: CursorEffectSettings) => void;
  /** Take every stroke off the picture being edited. */
  onDrawingsClear?: () => void;
  /** Round the output canvas corners by this share of its shorter side. */
  onFrameRadiusChange?: (radius: number) => void;
  onFrameReset?: () => void;
  /** Size the output canvas to what a field asked for. */
  onFrameSizeChange?: (size: { height?: number; width?: number }) => void;
  /** Draw every shortcut with these settings. */
  onKeyboardEffectsChange?: (settings: Partial<KeyboardEffectSettings>) => void;
  /** Put the shortcuts back where the recording draws them by default. */
  onKeyboardPositionReset?: () => void;
  /** Put every shortcut's own placement away, the global one left as it is. */
  onKeyboardShortcutsResetAll?: () => void;
  /** Bring back every shortcut deleted from the timeline. */
  onKeyboardShortcutsRestore?: () => void;
  /** Make the auto zooms again, the scenes of your own left as they are. */
  onSceneAutoZoom?: () => void;
  /** Take every auto zoom off the timeline, the scenes of your own left as
   * they are. */
  onSceneAutoZoomsClear?: () => void;
  /** Make the scene under the playhead custom, or add a custom scene there
   * where there is none. */
  onSceneCustomize?: () => void;
  /** Delete the scene under the playhead. */
  onSceneDelete?: () => void;
  /** Zoom the scene under the playhead into its selected pane, or move the
   * part of the pane it shows. */
  onSceneFramingChange?: (framing: Partial<SceneFraming>) => void;
  /** Show the whole screen and camera again in the scene under the playhead. */
  onSceneFramingReset?: () => void;
  /** Trade the panes of the custom scene under the playhead. */
  onScenePanesSwap?: () => void;
  /** Give the scene under the playhead this preset, or add a scene there
   * with it where there is none. */
  onScenePresetChoose?: (preset: RecordingScenePreset) => void;
  /** Round the selected pane's corners in the scene under the playhead. */
  onSceneRadiusChange?: (radius: number) => void;
  /** Lay the scene under the playhead out as this template, or add a scene
   * there laid out so where there is none. */
  onSceneTemplateChoose?: (template: SceneTemplate) => void;
  /** Forget the template with this id. */
  onSceneTemplateRemove?: (id: string) => void;
  /** Keep the custom scene under the playhead as a template. */
  onSceneTemplateSave?: () => void;
  /** Change the options of the preset under the playhead. */
  onSceneVariantChange?: (variant: SceneVariant) => void;
  /** Cast the selected layer's shadow onto the canvas, or take it away. */
  onSelectionDropShadowChange?: (dropShadow: boolean) => void;
  /** Pad the selected layer by this many output pixels on each side. */
  onSelectionInsetChange?: (inset: number) => void;
  /** Place the selection at the size and position a field asked for. */
  onSelectionPlacementChange?: (placement: SelectionPlacementPatch) => void;
  /** Round the selected layer's corners by this share of its shorter side. */
  onSelectionRadiusChange?: (radius: number) => void;
  /** Put the layer's content in the middle of its padded frame. */
  onSelectionRecenter?: () => void;
  onSelectionReset?: () => void;
  /** Give every shortcut the selected one's size and position. */
  onShortcutApplyToAll?: () => void;
  /** Draw the selected shortcut at this size and centre, all in percent. */
  onShortcutPlacementChange?: (
    placement: NonNullable<ToolPanelPatch["shortcutPlacement"]>,
  ) => void;
  /** Put the selected shortcut back where the recording drew it. */
  onShortcutReset?: () => void;
};
