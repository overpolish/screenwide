// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AnnotationStyle,
  ImageArt,
  TextSpan,
} from "../annotations/annotations";
import { SceneRotation } from "../layer-rotation";

import type { TextStyle } from "../../../bindings/TextStyle";
import type { AnnotationKind } from "../../../components/shared/annotation-style/types";

/**
 * What the selection panel shows for a placed layer: the selected layer, its
 * size and position in output pixels, and the source it was captured at, which
 * is what a reset puts it back to.
 */
export type ToolPanelLayerSelection = {
  /** Whether this layer casts a shadow onto the canvas behind it. */
  dropShadow: boolean;
  height: number;
  /** How far the padded frame runs past the layer's own picture, in output
   * pixels, on each side. */
  inset: number;
  /** As far as the padding may be taken: the layer's shorter side. */
  insetMaximum: number;
  /** Which of the workspace's layers this is, for wording that fits it. */
  kind: "camera" | "layer" | "primary";
  label: string;
  /** How rounded the layer's corners are, as a share of its shorter side,
   * 0 to 50. The same number the corner drag in the preview sets. */
  radius: number;
  sourceHeight: number;
  sourceWidth: number;
  width: number;
  x: number;
  y: number;
  /** The camera's effects, offered for the camera. */
  cameraEffects?: ToolPanelCameraEffects;
  /** How the layer is turned in depth, where it can be turned: the screen of
   * a recording and a screenshot's layers. */
  rotation?: SceneRotation;
};

/**
 * What the selection panel shows for a recorded audio track.
 *
 * An audio track is neither placed nor drawn: it is heard, so what there is
 * to set for it is how loud it is played back, in decibels against the level
 * it was recorded at, and for a microphone how clean the speech in it is.
 */
export type ToolPanelAudioSelection = {
  /** How much the track is lifted or lowered, 0 being the recorded level. */
  decibels: number;
  kind: "audio";
  /** The track's own name: "Microphone", "System audio", or "Audio" where the
   * recording did not say. */
  label: string;
  /** The speech tools, offered only for a microphone: other tracks have no
   * speech to find pauses in or noise to take out from under. */
  microphone?: ToolPanelMicrophone;
};

/**
 * The camera's effects: its background taken away, blurred or replaced by a
 * picture, and Enhance. They are drawn live; what each needs made first, the
 * matte or the measure Enhance corrects the colour by, is made once, and a
 * tool is `working` while it is, which can take a while the first time.
 */
export type ToolPanelCameraEffects = {
  background: "blurred" | "image" | "kept" | "transparent";
  /** The camera sharpened, enlarged along its edges where drawn larger, and
   * its white balance and exposure corrected with a natural look. The
   * colour waits for the recording's measure, its `colour` work. */
  enhance: boolean;
  /** Whether a picture has been chosen for the background. */
  hasImage: boolean;
  /** The model taking the background away needs, and how far it is on this
   * computer; `null` until the app has said. */
  model: ToolPanelModel | null;
  /** Objects picked to fix the matte, once the background is taken away;
   * `null` before. */
  objects: ToolPanelCameraObjects | null;
  /** How far each tool's work has got, 0 to 1, while it is working. */
  progress: Record<"background" | "colour", number>;
  working: Record<"background" | "colour", boolean>;
};

/**
 * Objects picked in the camera where the matte has them wrong, each kept
 * with the person or removed with the room. Each is picked in one frame and
 * followed through the recording, which takes a while, then laid into the
 * matte: `working`, at `progress`, while that is.
 */
export type ToolPanelCameraObjects = {
  /** The object chosen while picking, which the bin takes away. */
  chosen: number | null;
  /** Whether objects are being picked on the preview. */
  isPicking: boolean;
  /** The model following them needs, and how far it is on this computer. */
  model: ToolPanelModel | null;
  /** Each object picked: followed through the recording, being followed
   * and how far, 0 to 1, or waiting its turn; `painted` where a line was
   * painted over it rather than a click. */
  picks: {
    followed: boolean;
    following: number | null;
    id: number;
    keep: boolean;
    painted: boolean;
  }[];
  progress: number;
  working: boolean;
};

/** Where Remove silences stands for the recording's microphone. One state,
 * shown both in the microphone's Select panel and in the Transcript panel. */
export type ToolPanelSilences = {
  /** How many pauses Remove cut that are still cut. */
  count: number;
  /** How much shorter those cuts make the recording. */
  durationMs: number;
  /** How far listening has got, 0 to 1, while `status` is `finding`;
   * `null` while it is not known to need listening, as when the track
   * was listened through before and finding takes no time. */
  progress: number | null;
  /** `finding` while the track is listened through, and `none-found` after
   * a Remove that found no pause long enough to cut. */
  status: "finding" | "idle" | "none-found";
};

/**
 * Where the microphone's speech tools stand. Reduce noise and Remove
 * silences work from the same listen through the track for speech, which
 * runs once, the first time either asks. Studio sound stands in for Reduce
 * noise and Vocal cleanup while it is on and its model is downloaded.
 */
export type ToolPanelMicrophone = {
  /** The voice brought to a steady, standard loudness, and the system audio
   * made to make way for it while it speaks: `cleaning` while that work is
   * done for the track as it is heard, once for each way it can be heard. */
  autoVolume: "cleaning" | "off" | "on";
  /** How far measuring has got, 0 to 1, while `autoVolume` is `cleaning`. */
  autoVolumeProgress: number;
  /** Studio ducking, for a recording with system audio, `null` without: the
   * singing in the system audio moved out of the voice's way while it
   * speaks, by a downloaded model, where the rest only makes way. `cleaning`
   * while the singing is found, which happens once and can take a while. */
  ducking: {
    model: ToolPanelModel | null;
    /** How far finding the singing has got, 0 to 1, while `cleaning`. */
    progress: number;
    state: "cleaning" | "off" | "on";
  } | null;
  /** Everything but the voice taken out of the track: `cleaning` while the
   * track is cleaned, which happens once and can take a while. */
  noise: "cleaning" | "off" | "on";
  /** How far cleaning has got, 0 to 1, while `noise` is `cleaning`. */
  noiseProgress: number;
  silences: ToolPanelSilences;
  /** The voice rebuilt as a studio recording by a downloaded model, in place
   * of Reduce noise and Vocal cleanup: `cleaning` while the track is
   * rebuilt, which happens once and can take a while. */
  studio: "cleaning" | "off" | "on";
  /** The model Studio sound needs, and how far it is on this computer;
   * `null` until the app has said. */
  studioModel: ToolPanelModel | null;
  /** How far rebuilding has got, 0 to 1, while `studio` is `cleaning`. */
  studioProgress: number;
  /** The voice cleaned up: mouth clicks and harsh "s" sounds softened, its
   * tone eased and its level evened out. `cleaning` while that is done to
   * the track as it is heard, once each way Reduce noise is set. */
  voice: "cleaning" | "off" | "on";
  /** How far cleaning up has got, 0 to 1, while `voice` is `cleaning`. */
  voiceProgress: number;
};

/** A model a microphone switch needs, and how far it is on this computer.
 * `progress` runs 0 to 1 while it downloads. */
export type ToolPanelModel = {
  progress: number;
  sizeBytes: number;
  status: "available" | "downloaded" | "downloading";
};

export type ToolPanelSelection =
  ToolPanelAudioSelection | ToolPanelLayerSelection;

/**
 * What the annotation panel shows: the annotation the preview has in hand, and
 * the dress it is drawn in.
 *
 * An annotation is neither placed nor sized in output pixels - it is drawn, in
 * the source's own space, by the compositor - so there is nothing here of where
 * it sits: the picture itself is where it is moved, and the panel only ever
 * says what it looks like. `kind` is what the panel offers controls for: an
 * arrow has heads and a stroke, a counter a disc.
 */
export type ToolPanelAnnotation = {
  /** Whether this annotation draws itself in at the start of its clip and
   * undraws at the end. A screenshot has no clip to animate over, so the panel
   * shows it only in the recording editor. */
  animated: boolean;
  id: string;
  kind: AnnotationKind;
  style: AnnotationStyle;
  /** A counter's aim, in radians clockwise from east. Absent on an arrow,
   * which is aimed by its own two ends, and on an image, which is turned. */
  angle?: number;
  /** The words a placed text sets apart, each by its bytes in its UTF-8.
   * Absent on anything else. */
  emphasis?: TextSpan[];
  /** The picture an image shows, or the next image will. */
  image?: ImageArt;
  /** Whether this is no annotation yet but the dress the tool in hand draws
   * its next one in. Its edits dress that one, and what only a drawn
   * annotation can do - turning it round, drawing it again - is not offered. */
  isDraft?: boolean;
  /** The look a text is dressed in, or the next text will be once one has
   * been chosen. Absent on anything else. */
  look?: TextStyle;
  /** How a placed image is turned, in degrees about each axis, as the
   * Rotation row's dials show it. Absent on anything else. */
  rotation?: SceneRotation;
  /** Whether a placed image sways over its clip. Absent on anything else. */
  sway?: boolean;
  /** What a placed text says. Absent on anything else. */
  text?: string;
};
