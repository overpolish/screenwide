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
import { KeyboardEffectSettings } from "../types";

import { BackgroundPreset } from "./frame/background/background";
import { ToolPanelExportPatch } from "./tool-panel-export";
import { TranscriptRequest } from "./tool-panel-transcript";

import type { ToolPanelSnapshot } from "./tool-panel-store";
import type { TextStyle } from "../../../bindings/TextStyle";
import type { AnnotationPinAction } from "../island/annotation/annotation-pin";
import type { CropAspect } from "../island/crop-aspect-choice";

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
  /** Turn the chosen focus into a spotlight, or the spotlight into a focus:
   * one pen with two looks, blurred and dimmed. */
  annotationKind?: "focus" | "spotlight";
  /** Dress the chosen text in this look, which the next text wears too. */
  annotationLook?: TextStyle;
  /** Pin the chosen annotation to its content, let it go, say its content
   * is out of view or back at the playhead, or take its corrections away:
   * the menu's Tracking actions. */
  annotationPin?: AnnotationPinAction;
  /** Turn the chosen image, in degrees about each axis, a field at a time:
   * in depth about X and Y, flat about Z. */
  annotationRotation?: Partial<SceneRotation>;
  /** Dress the chosen annotation, a field at a time so a panel never has to
   * send the whole style back. Each one also becomes the next arrow's default.
   */
  annotationStyle?: Partial<AnnotationStyle>;
  /** Set how many times the chosen lens enlarges, one of `LENS_ZOOMS`. The
   * zoom is the lens's own shape rather than its dress, and the next lens is
   * drawn at it too. */
  annotationZoom?: number;
  /** Bring the selected microphone to a steady loudness, or play it as
   * recorded. */
  autoVolume?: boolean;
  /** Make the auto zooms again, the scenes of your own left as they are. */
  autoZoomScenes?: true;
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
  /** Give the crop a shape to keep as its handles are dragged, cutting it to
   * that shape about its middle, or let it go free. */
  cropAspect?: CropAspect;
  /** Cut what is chosen on the strip, a beat, lull, moment or drawn range,
   * or several beats and lulls, and its time with it. */
  cutStripItem?: true;
  /** Delete every chosen annotation, in one edit. */
  deleteAnnotations?: true;
  /** Delete the shortcuts chosen in the strip's shortcut lane. */
  deleteChosenShortcuts?: true;
  /** Delete the scene under the playhead. */
  deleteScene?: true;
  /** Let go of what is chosen on the strip. */
  deselectStripItem?: true;
  /** Download the model taking the camera's background away needs. */
  downloadCameraBackground?: true;
  /** Download the model following the camera's objects needs. */
  downloadCameraObjects?: true;
  /** Download the model Studio ducking separates the singing with. */
  downloadStudioDucking?: true;
  /** Download the model Studio sound rebuilds the microphone with. */
  downloadStudioSound?: true;
  /** Something the Export island asks for. */
  exportPatch?: ToolPanelExportPatch;
  /** Round the output canvas corners by this share of its shorter side. */
  frameRadius?: number;
  /** Size the output canvas, leaving what is in it where it sits. */
  frameSize?: { height?: number; width?: number };
  /** The shortcut settings every shortcut is drawn with, changed a field at a
   * time so a panel never has to send the whole group back. */
  keyboardEffects?: Partial<KeyboardEffectSettings>;
  /** Have the lull chosen on the strip squeeze with the rest, or keep it
   * as recorded. */
  lullPlay?: "keep" | "squeeze";
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
  /** Show the whole source again, the committed crop taken away. */
  resetCrop?: true;
  /** Put the canvas back to the source size, refitting what is in it. */
  resetFrame?: true;
  /** Put the selected layer back where a fresh project places it, keeping
   * what it shows. */
  resetPlacement?: true;
  /** Bring back every shortcut deleted from the timeline. */
  restoreShortcuts?: true;
  /** Turn the chosen annotation round, so its head points the other way, or
   * mirror the chosen image. */
  reverseAnnotation?: true;
  /** Keep a colour of your own, so it is on offer next time. */
  saveAnnotationColor?: string;
  /** Keep the background being shown under a name. */
  savePreset?: BackgroundPreset;
  /** Keep the custom scene under the playhead as a template. */
  saveSceneTemplate?: true;
  /** Have the camera in the scene under the playhead follow the face, or
   * stop following it. */
  sceneFollowFace?: boolean;
  /** Zoom the scene under the playhead into its selected pane, or move the
   * part of the pane it shows, a field at a time. */
  sceneFraming?: Partial<SceneFraming>;
  /** The pane `sceneFraming` reaches: the selected one where unset. */
  sceneFramingPane?: "camera" | "screen";
  /** Pad the screen in the scene under the playhead by this share of its
   * box's shorter side on each side. */
  sceneInset?: number;
  /** Put the pane of the scene under the playhead back in the box its
   * layout gives it, keeping what it shows. */
  scenePlacementReset?: "camera" | "screen";
  /** Give the scene under the playhead this preset, or add a scene there
   * with it where there is none. */
  scenePreset?: RecordingScenePreset;
  /** Round the selected pane's corners in the scene under the playhead by
   * this share of its shorter side, in percent. */
  sceneRadius?: number;
  /** Crop the screen in the scene under the playhead to the content found
   * in it, in the middle of where its box was. */
  sceneRecenter?: true;
  /** Turn the screen in depth in the scene under the playhead, in degrees
   * about each axis, a field at a time. */
  sceneRotation?: Partial<SceneRotation>;
  /** Change the options of the preset under the playhead, a field at a time. */
  sceneVariant?: SceneVariant;
  /** Cast the selected layer's shadow onto the canvas, or take it away. */
  selectionDropShadow?: boolean;
  /** Have the selected camera follow the face, outside every scene and in
   * each scene that says nothing of its own, or stop following it. */
  selectionFollowFace?: boolean;
  /** Pad the selected layer by this many output pixels on each side. */
  selectionInset?: number;
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
  /** Hold the recorded track with this id in the export and the preview,
   * or leave it out. */
  trackEnabled?: { isEnabled: boolean; trackId: string };
  /** Play the audio track with this id this much louder or quieter than it
   * was recorded, in decibels, whichever track is selected. */
  trackVolume?: { decibels: number; trackId: string };
  /** Something the Transcript panel asks for: never a setting it holds. */
  transcriptRequest?: TranscriptRequest;
  /** Trim the lull chosen on the strip to a breath: all but its first
   * moment is cut. */
  trimLullToBreath?: true;
  /** Clean up the selected microphone's voice, or put it back as it was. */
  vocalCleanup?: boolean;
};
