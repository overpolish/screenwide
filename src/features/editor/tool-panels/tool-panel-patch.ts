// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { BackgroundPreset } from "../../../components/shared/background-picker/background";
import { ImagePlayChange } from "../annotations/annotation-channel";
import { AnnotationStyle, ImageArt } from "../annotations/annotations";
import { SceneTemplate } from "../recording/scenes/recording-scene-template";
import { SceneVariant } from "../recording/scenes/recording-scene-variant";
import {
  RecordingScenePreset,
  SceneFraming,
} from "../recording/scenes/recording-scenes";
import { KeyboardEffectSettings } from "../types";

import { SelectionPlacementPatch } from "./selection-placement";

import type { ToolPanelSnapshot } from "./tool-panel-store";

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
  /** Give the chosen image this picture, which the next image shows too.
   * The picture is the annotation's own property rather than part of its
   * dress. */
  annotationImage?: ImageArt;
  /** Change how the chosen moving image plays: the frame it shows or
   * starts on, and whether a recording plays it through once. */
  annotationImagePlay?: ImagePlayChange;
  /** Sway the chosen image over its clip, or stand it still. Turning it on
   * gives it a sway of its own; the dice gives it another. */
  annotationImageSway?: boolean;
  /** Dress the chosen annotation, a field at a time so a panel never has to
   * send the whole style back. Each one also becomes the next arrow's default.
   */
  annotationStyle?: Partial<AnnotationStyle>;
  /** Put every shortcut back where the recording drew it. */
  applyShortcutToAll?: true;
  /** Play the selected audio track this much louder or quieter than it was
   * recorded, in decibels. */
  audioVolume?: number;
  /** Make the auto zooms again, the scenes of your own left as they are. */
  autoZoomScenes?: true;
  /** Draw the camera into the screen's video, or save it as a file of its
   * own. */
  bakeCamera?: boolean;
  /** Lay the scene under the playhead out as this template. */
  chooseSceneTemplate?: SceneTemplate;
  /** Take every auto zoom off the timeline, the scenes of your own left as
   * they are. */
  clearAutoZooms?: true;
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
  /** Take the room's steady noise out of the selected microphone, or put it
   * back. */
  reduceNoise?: boolean;
  /** Forget a colour of your own, by the colour itself. */
  removeAnnotationColor?: string;
  /** Forget a saved background, by its id. */
  removePreset?: string;
  /** Forget a scene template, by its id. */
  removeSceneTemplate?: string;
  /** Cut the long pauses out of the selected microphone's recording where
   * nothing happens on screen. */
  removeSilences?: true;
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
  /** Bring back every pause Remove cut that is still cut. */
  restoreSilences?: true;
  /** Turn the chosen annotation round, so its head points the other way, or
   * mirror the chosen image. */
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
  /** Trade the panes of the custom scene under the playhead: each takes the
   * other's box and place in the order. */
  swapScenePanes?: true;
};
