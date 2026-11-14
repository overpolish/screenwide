// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ImagePlayChange } from "../annotations/annotation-channel";
import {
  AnnotationStyle,
  ImageArt,
  TextSpan,
} from "../annotations/annotations";
import { CameraEffectsChange } from "../recording/camera/use-camera-effects";
import { CaptionsPatch } from "../recording/captions/recording-captions";
import { CaptionDefaultsPatch } from "../recording/captions/use-text-templates";
import { SceneTemplate } from "../recording/scenes/recording-scene-template";
import { SceneVariant } from "../recording/scenes/recording-scene-variant";
import {
  RecordingScenePreset,
  SceneFraming,
  SceneRotation,
} from "../recording/scenes/recording-scenes";
import { CursorEffectSettings, KeyboardEffectSettings } from "../types";

import { Background, BackgroundPreset } from "./frame/background/background";
import { TranscriptRequest } from "./tool-panel-transcript";

import type { ToolPanelExportPatch } from "./tool-panel-export";
import type { TextStyle } from "../../../bindings/TextStyle";
import type { AnnotationPinAction } from "../island/annotation/annotation-pin";
import type { AudioClipChange } from "../island/audio-clip/audio-clip-island";
import type { CropAspect } from "../island/crop-aspect-choice";
import type { SoundAddition } from "../island/library/library-panel";

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
  /** Set apart the chosen text's words, and no others. */
  onAnnotationEmphasisChange?: (emphasis: TextSpan[]) => void;
  /** Give the chosen image, or the next one, another picture. */
  onAnnotationImageChange?: (art: ImageArt) => void;
  /** Change how the chosen moving image plays. */
  onAnnotationImagePlayChange?: (play: ImagePlayChange) => void;
  /** Sway the chosen image over its clip, or stand it still. */
  onAnnotationImageSwayChange?: (sway: boolean) => void;
  /** Turn the chosen focus into a spotlight, or the spotlight into a focus. */
  onAnnotationKindChange?: (kind: "focus" | "spotlight") => void;
  /** Turn the chosen annotation round, through the same commit path the drag on
   * the picture uses. */
  onAnnotationLookChange?: (look: TextStyle) => void;
  /** Pin the chosen annotation to its content, let it go, say its content
   * is out of view or back at the playhead, or take its corrections away. */
  onAnnotationPin?: (action: AnnotationPinAction) => void;
  onAnnotationReverse?: () => void;
  /** Turn the chosen image, a field at a time, in degrees about each axis. */
  onAnnotationRotationChange?: (rotation: Partial<SceneRotation>) => void;
  /** Take away the chosen annotation's own place in the scene at the
   * playhead. */
  onAnnotationSceneReset?: () => void;
  /** Lay the chosen redaction's blocks out again, draw the chosen hand-drawn
   * highlight's stroke again, or sway the chosen image another way, from a
   * fresh seed. */
  onAnnotationShuffle?: () => void;
  /** Dress the chosen annotation, a field at a time, through the same commit
   * path the drag on the picture uses. */
  onAnnotationStyleChange?: (style: Partial<AnnotationStyle>) => void;
  /** Set how many times the chosen lens, or the next one, enlarges. */
  onAnnotationZoomChange?: (zoom: number) => void;
  /** Delete every chosen annotation, in one edit. */
  onAnnotationsDelete?: () => void;
  /** Change the song or sound effect chosen on the strip, delete it or give
   * it another sound. */
  onAudioClipChange?: (change: AudioClipChange) => void;
  /** Bring the selected microphone to a steady loudness, or play it as
   * recorded. */
  onAutoVolumeChange?: (enabled: boolean) => void;
  /** Fill the workspace's canvas with this background. */
  onBackgroundChange?: (background: Background) => void;
  onBackgroundPresetRemove?: (id: string) => void;
  onBackgroundPresetSave?: (preset: BackgroundPreset) => void;
  /** Ask for a picture to put behind the person on camera. */
  onCameraBackgroundChoose?: () => void;
  /** Download the model taking the camera's background away needs. */
  onCameraBackgroundDownload?: () => void;
  /** Change the camera's effects, the rest kept as they are. */
  onCameraEffectsChange?: (change: CameraEffectsChange) => void;
  /** Take the object picked with this id out of the camera's picks. */
  onCameraObjectRemove?: (id: number) => void;
  /** Go to the frame the object with this id was picked in, and pick there. */
  onCameraObjectShow?: (id: number) => void;
  /** Download the model following the camera's objects needs. */
  onCameraObjectsDownload?: () => void;
  /** Start picking objects in the camera frame the preview shows, or stop. */
  onCameraObjectsPick?: (isPicking: boolean) => void;
  /** Save a caption template's style as its default, or choose the template
   * new captions start from. */
  onCaptionDefaultsChange?: (patch: CaptionDefaultsPatch) => void;
  /** Show the captions or set them otherwise, as one edit. */
  onCaptionsChange?: (patch: CaptionsPatch) => void;
  /** Delete the shortcuts chosen in the strip's shortcut lane. */
  onChosenShortcutsDelete?: () => void;
  /** Give the crop a shape to keep, or let it go free. */
  onCropAspectChange?: (aspect: CropAspect) => void;
  /** Show the whole source again, the committed crop taken away. */
  onCropReset?: () => void;
  onCursorEffectsChange?: (settings: CursorEffectSettings) => void;
  /** Do what the Export island asks: a setting, or a step of the export. */
  onExportPatch?: (patch: ToolPanelExportPatch) => void;
  /** Round the output canvas corners by this share of its shorter side. */
  onFrameRadiusChange?: (radius: number) => void;
  onFrameReset?: () => void;
  /** Size the output canvas to what a field asked for. */
  onFrameSizeChange?: (size: { height?: number; width?: number }) => void;
  /** Draw every shortcut with these settings. */
  onKeyboardEffectsChange?: (settings: Partial<KeyboardEffectSettings>) => void;
  /** Bring back every shortcut deleted from the timeline. */
  onKeyboardShortcutsRestore?: () => void;
  /** Place an image showing a picture from the library in the middle of
   * the layer in hand. */
  onLibraryImage?: (art: ImageArt) => void;
  /** Have the lull chosen on the strip squeeze with the rest, or keep it
   * as recorded. */
  onLullPlayChange?: (play: "keep" | "squeeze") => void;
  /** Trim the lull chosen on the strip to a breath. */
  onLullTrimToBreath?: () => void;
  /** Take the room's steady noise out of the selected microphone, or put it
   * back. */
  onReduceNoiseChange?: (reduce: boolean) => void;
  /** Make the auto zooms again, the scenes of your own left as they are. */
  onSceneAutoZoom?: () => void;
  /** Take every auto zoom off the timeline, the scenes of your own left as
   * they are. */
  onSceneAutoZoomsClear?: () => void;
  /** Give the scene under the playhead a background of its own, or with
   * null have it show the recording's. */
  onSceneBackgroundChange?: (background: Background | null) => void;
  /** Show or hide the cursor in the scene under the playhead over the
   * recording's own choice, or with null have it do as the recording does. */
  onSceneCursorChange?: (shown: boolean | null) => void;
  /** Delete the scene under the playhead. */
  onSceneDelete?: () => void;
  /** Have the camera in the scene under the playhead follow the face, or
   * stop following it. */
  onSceneFollowFaceChange?: (follow: boolean) => void;
  /** Zoom the scene under the playhead into its selected pane, or move the
   * part of the pane it shows. */
  onSceneFramingChange?: (
    framing: Partial<SceneFraming>,
    pane?: "camera" | "screen",
  ) => void;
  /** Pad the screen in the scene under the playhead by this share of its
   * box's shorter side. */
  onSceneInsetChange?: (inset: number) => void;
  /** Trade the panes of the custom scene under the playhead. */
  onScenePanesSwap?: () => void;
  /** Put the pane of the scene under the playhead back in the box its
   * layout gives it. */
  onScenePlacementReset?: (pane: "camera" | "screen") => void;
  /** Give the scene under the playhead this preset, or add a scene there
   * with it where there is none. */
  onScenePresetChoose?: (preset: RecordingScenePreset) => void;
  /** Round the selected pane's corners in the scene under the playhead. */
  onSceneRadiusChange?: (radius: number) => void;
  /** Crop the screen in the scene under the playhead to its content, in the
   * middle of its box. */
  onSceneRecenter?: () => void;
  /** Turn the screen in depth in the scene under the playhead, a field at a
   * time. */
  onSceneRotationChange?: (rotation: Partial<SceneRotation>) => void;
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
  /** Have the selected camera follow the face, or stop following it. */
  onSelectionFollowFaceChange?: (follow: boolean) => void;
  /** Pad the selected layer by this many output pixels on each side. */
  onSelectionInsetChange?: (inset: number) => void;
  /** Put the selected layer back where a fresh project places it. */
  onSelectionPlacementReset?: () => void;
  /** Round the selected layer's corners by this share of its shorter side. */
  onSelectionRadiusChange?: (radius: number) => void;
  /** Put the layer's content in the middle of its padded frame. */
  onSelectionRecenter?: () => void;
  /** Turn the selected layer in depth, a field at a time. */
  onSelectionRotationChange?: (rotation: Partial<SceneRotation>) => void;
  /** Add a song or sound effect from the library to the timeline. */
  onSoundAdd?: (addition: SoundAddition) => void;
  /** Cut what is chosen on the strip, and its time with it. */
  onStripItemCut?: () => void;
  /** Let go of what is chosen on the strip. */
  onStripItemDeselect?: () => void;
  /** Move the singing in the system audio out of the selected microphone's
   * way while it speaks, or let the system audio make way as a whole. */
  onStudioDuckingChange?: (enabled: boolean) => void;
  /** Download the model Studio ducking separates the singing with. */
  onStudioDuckingDownload?: () => void;
  /** Rebuild the selected microphone as a studio recording, or bring back
   * Reduce noise and Vocal cleanup. */
  onStudioSoundChange?: (enabled: boolean) => void;
  /** Download the model Studio sound rebuilds the microphone with. */
  onStudioSoundDownload?: () => void;
  /** Hold the recorded track `trackId` in the export and the preview, or
   * leave it out. */
  onTrackEnabledChange?: (trackId: string, isEnabled: boolean) => void;
  /** Play the audio track `trackId` this much louder or quieter than it was
   * recorded, in decibels. */
  onTrackVolumeChange?: (trackId: string, decibels: number) => void;
  /** Do what the Transcript panel asks. */
  onTranscriptRequest?: (request: TranscriptRequest) => void;
  /** Clean up the selected microphone's voice, or put it back as it was. */
  onVocalCleanupChange?: (enabled: boolean) => void;
};
