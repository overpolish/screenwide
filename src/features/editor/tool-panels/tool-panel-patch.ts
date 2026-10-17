// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { BackgroundPreset } from "../../../components/shared/background-picker/background";
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
import { KeyboardEffectSettings } from "../types";

import { SelectionPlacementPatch } from "./selection-placement";
import { TranscriptRequest } from "./tool-panel-transcript";

import type { ToolPanelSnapshot } from "./tool-panel-store";
import type { TextStyle } from "../../../bindings/TextStyle";

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
  /** Set apart the chosen text's words, each by its bytes in the text's
   * UTF-8, and no others. */
  annotationEmphasis?: TextSpan[];
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
  /** Dress the chosen text in this look, which the next text wears too. */
  annotationLook?: TextStyle;
  /** Turn the chosen image, in degrees about each axis, a field at a time:
   * in depth about X and Y, flat about Z. */
  annotationRotation?: Partial<SceneRotation>;
  /** Dress the chosen annotation, a field at a time so a panel never has to
   * send the whole style back. Each one also becomes the next arrow's default.
   */
  annotationStyle?: Partial<AnnotationStyle>;
  /** Play the selected audio track this much louder or quieter than it was
   * recorded, in decibels. */
  audioVolume?: number;
  /** Bring the selected microphone to a steady loudness, or play it as
   * recorded. */
  autoVolume?: boolean;
  /** Make the auto zooms again, the scenes of your own left as they are. */
  autoZoomScenes?: true;
  /** Draw the camera into the screen's video, or save it as a file of its
   * own. */
  bakeCamera?: boolean;
  /** Change the camera's effects, a field at a time: what is behind the
   * person, and colour correction. */
  cameraEffects?: CameraEffectsChange;
  /** Change what new captions start as: a template and its look, the
   * layout options set by hand, the rules for what they say, or a template
   * put back to how it ships. */
  captionDefaults?: CaptionDefaultsPatch;
  /** Show the captions or set them otherwise. */
  captions?: CaptionsPatch;
  /** Ask for a picture to put behind the person on camera. */
  chooseCameraBackground?: true;
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
  /** Download the model taking the camera's background away needs. */
  downloadCameraBackground?: true;
  /** Download the model following the camera's objects needs. */
  downloadCameraObjects?: true;
  /** Download the model Studio ducking separates the singing with. */
  downloadStudioDucking?: true;
  /** Download the model Studio sound rebuilds the microphone with. */
  downloadStudioSound?: true;
  /** Round the output canvas corners by this share of its shorter side. */
  frameRadius?: number;
  /** Size the output canvas, leaving what is in it where it sits. */
  frameSize?: { height?: number; width?: number };
  /** The shortcut settings every shortcut is drawn with, changed a field at a
   * time so a panel never has to send the whole group back. */
  keyboardEffects?: Partial<KeyboardEffectSettings>;
  /** Start picking objects in the camera on the preview, or stop. */
  pickCameraObjects?: boolean;
  /** Put the selected layer's content in the middle of its padded frame. */
  recenterSelection?: true;
  /** Take the room's steady noise out of the selected microphone, or put it
   * back. */
  reduceNoise?: boolean;
  /** Forget a colour of your own, by the colour itself. */
  removeAnnotationColor?: string;
  /** Take the object picked with this id out of the camera's picks. */
  removeCameraObject?: number;
  /** Forget a saved background, by its id. */
  removePreset?: string;
  /** Forget a scene template, by its id. */
  removeSceneTemplate?: string;
  /** Cut the long pauses out of the selected microphone's recording where
   * nothing happens on screen. */
  removeSilences?: true;
  /** Show the whole source again, the committed crop taken away. */
  resetCrop?: true;
  /** Put the canvas back to the source size, refitting what is in it. */
  resetFrame?: true;
  /** Show the whole screen and camera again in the scene under the playhead. */
  resetSceneFraming?: true;
  /** Put the selection's size and position back to its source framing. */
  resetSelection?: true;
  /** Bring back every shortcut deleted from the timeline. */
  restoreShortcuts?: true;
  /** Bring back every pause Remove cut that is still cut. */
  restoreSilences?: true;
  /** Turn the chosen annotation round, so its head points the other way, or
   * mirror the chosen image. */
  reverseAnnotation?: true;
  /** Keep a colour of your own, so it is on offer next time. */
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
  /** Turn the screen in depth in the scene under the playhead, in degrees
   * about each axis, a field at a time. */
  sceneRotation?: Partial<SceneRotation>;
  /** Change the options of the preset under the playhead, a field at a time. */
  sceneVariant?: SceneVariant;
  /** Cast the selected layer's shadow onto the canvas, or take it away. */
  selectionDropShadow?: boolean;
  /** Pad the selected layer by this many output pixels on each side. */
  selectionInset?: number;
  selectionOutput?: SelectionPlacementPatch;
  /** Round the selected layer's corners by this share of its shorter side. */
  selectionRadius?: number;
  /** Turn the selected layer in depth, in degrees about each axis, a field
   * at a time. */
  selectionRotation?: Partial<SceneRotation>;
  /** Go to the frame the object with this id was picked in, and pick there
   * with it chosen. */
  showCameraObject?: number;
  /** Lay the chosen redaction's blocks out again, or draw the chosen
   * hand-drawn highlight's stroke again, from a fresh seed. */
  shuffleAnnotation?: true;
  /** Move the singing in the selected microphone's system audio out of its
   * way while it speaks, or let the system audio make way as a whole. */
  studioDucking?: boolean;
  /** Rebuild the selected microphone as a studio recording, in place of
   * Reduce noise and Vocal cleanup, or bring them back. */
  studioSound?: boolean;
  /** Trade the panes of the custom scene under the playhead: each takes the
   * other's box and place in the order. */
  swapScenePanes?: true;
  /** Something the Transcript panel asks for: never a setting it holds. */
  transcriptRequest?: TranscriptRequest;
  /** Clean up the selected microphone's voice, or put it back as it was. */
  vocalCleanup?: boolean;
};
