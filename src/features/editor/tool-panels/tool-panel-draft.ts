// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { patchCaptions } from "../recording/captions/recording-captions";
import { EditorKind } from "../types";

import { SelectionPlacementPatch } from "./selection-placement";
import { ToolPanelPatch } from "./tool-panel-patch";
import { ToolPanelMicrophone, ToolPanelSnapshot } from "./tool-panel-store";

export type ToolPanelDraft = {
  seq: number;
  values: ToolPanelPatch;
  workspace: EditorKind;
};

/** Only the fields the panel is holding override the published placement, so a
 * drag in the preview keeps moving the rest of them under an open field. */
const placedSelection = (
  selection: ToolPanelSnapshot["selection"],
  placement: SelectionPlacementPatch,
) => {
  if (!selection || selection.kind === "audio") return selection;
  const next = { ...selection };
  for (const key of ["height", "width", "x", "y"] as const) {
    const value = placement[key];
    if (value !== undefined) next[key] = value;
  }
  return next;
};

/** A microphone switch as the panel shows it once flipped to `enabled`:
 * turned on, it reads as cleaning until the editor says its track is ready. */
const flippedSwitch = (
  state: ToolPanelMicrophone["noise"],
  enabled: boolean | undefined,
): ToolPanelMicrophone["noise"] => {
  if (enabled === undefined) return state;
  if (!enabled) return "off";
  return state === "on" ? "on" : "cleaning";
};

/** The same rule for the canvas and the crop: a width being typed does not pin
 * the height a drag on the picture is still moving. */
const sized = <Value extends { height: number; width: number }>(
  value: Value | null,
  size: { height?: number; width?: number },
) => {
  if (!value) return value;
  const next = { ...value };
  for (const key of ["height", "width"] as const) {
    const typed = size[key];
    if (typed !== undefined) next[key] = typed;
  }
  return next;
};

/** Only pending edits override the mirror; metadata stays live throughout. */
export function resolveToolPanelSnapshot(
  snapshot: ToolPanelSnapshot,
  draft: ToolPanelDraft | null,
  workspace: EditorKind,
): ToolPanelSnapshot {
  if (
    draft?.workspace !== workspace ||
    draft.seq <= (snapshot.acknowledgedSeq ?? 0)
  )
    return snapshot;
  // A reset is an action rather than a value: nothing of it is shown locally,
  // and the editor's answer arrives as the next published placement.
  const {
    annotationAngle,
    annotationAnimated,
    annotationEmphasis,
    annotationImageSway,
    annotationRotation,
    annotationStyle,
    audioVolume,
    autoVolume,
    autoZoomScenes: _autoZoomScenes,
    bakeCamera,
    cameraEffects,
    captionDefaults: _captionDefaults,
    captions,
    chooseCameraBackground: _chooseCameraBackground,
    chooseSceneTemplate: _chooseSceneTemplate,
    clearAutoZooms: _clearAutoZooms,
    clearDrawings: _clearDrawings,
    cropSize,
    customizeScene: _customizeScene,
    deleteAnnotations: _deleteAnnotations,
    deleteScene: _deleteScene,
    downloadCameraBackground: _downloadCameraBackground,
    downloadCameraObjects: _downloadCameraObjects,
    downloadStudioDucking: _downloadStudioDucking,
    downloadStudioSound: _downloadStudioSound,
    frameRadius,
    frameSize,
    keyboardEffects,
    pickCameraObjects: _pickCameraObjects,
    recenterSelection: _recenterSelection,
    reduceNoise,
    removeAnnotationColor: _removeAnnotationColor,
    removeCameraObject: _removeCameraObject,
    removePreset: _removePreset,
    removeSceneTemplate: _removeSceneTemplate,
    removeSilences: _removeSilences,
    resetCrop: _resetCrop,
    resetFrame: _resetFrame,
    resetSceneFraming: _resetSceneFraming,
    resetSelection: _resetSelection,
    restoreShortcuts: _restoreShortcuts,
    restoreSilences: _restoreSilences,
    saveAnnotationColor: _saveAnnotationColor,
    savePreset: _savePreset,
    saveSceneTemplate: _saveSceneTemplate,
    sceneFraming,
    scenePreset,
    sceneRadius,
    sceneRotation,
    sceneVariant,
    selectionDropShadow,
    selectionInset,
    selectionOutput,
    selectionRadius,
    selectionRotation,
    showCameraObject: _showCameraObject,
    studioDucking,
    studioSound,
    swapScenePanes: _swapScenePanes,
    transcriptRequest: _transcriptRequest,
    vocalCleanup,
    ...values
  } = draft.values;
  const resolved = {
    ...snapshot,
    ...values,
    // A shortcut setting is sent one field at a time, so the rest of the group
    // keeps whatever the editor last published for it.
    ...(keyboardEffects
      ? { keyboardEffects: { ...snapshot.keyboardEffects, ...keyboardEffects } }
      : {}),
    // So are the captions' settings, and the switch shows at once.
    captions:
      snapshot.captions && patchCaptions(snapshot.captions, captions ?? {}),
  };
  // A dragged inset holds its own value until the editor acknowledges it, so
  // the knob stays under the pointer rather than snapping back a frame.
  const placed =
    resolved.selection?.kind === "audio" ? null : resolved.selection;
  const held =
    placed && selectionInset !== undefined
      ? { ...placed, inset: selectionInset }
      : placed;
  const rounded =
    held && selectionRadius !== undefined
      ? { ...held, radius: selectionRadius }
      : held;
  const shadowed =
    rounded && selectionDropShadow !== undefined
      ? { ...rounded, dropShadow: selectionDropShadow }
      : rounded;
  // A dialled turn shows at once, as a scene's does.
  const turned =
    shadowed?.rotation && selectionRotation
      ? {
          ...shadowed,
          rotation: { ...shadowed.rotation, ...selectionRotation },
        }
      : shadowed;
  // The camera's effects change at once, as the microphone's switches do;
  // the bars arrive with the editor's answer.
  const layer =
    turned?.cameraEffects && cameraEffects
      ? {
          ...turned,
          cameraEffects: { ...turned.cameraEffects, ...cameraEffects },
        }
      : turned;
  // A dragged volume is held the same way: the knob stays where it was let go
  // of until the editor answers with the level it committed. The microphone
  // switches flip at once too.
  const audio =
    resolved.selection?.kind === "audio" ? resolved.selection : null;
  const flipping =
    reduceNoise !== undefined ||
    vocalCleanup !== undefined ||
    autoVolume !== undefined ||
    studioSound !== undefined ||
    studioDucking !== undefined;
  const microphone =
    audio?.microphone && flipping
      ? {
          ...audio.microphone,
          autoVolume: flippedSwitch(audio.microphone.autoVolume, autoVolume),
          ducking: audio.microphone.ducking && {
            ...audio.microphone.ducking,
            state: flippedSwitch(audio.microphone.ducking.state, studioDucking),
          },
          noise: flippedSwitch(audio.microphone.noise, reduceNoise),
          studio: flippedSwitch(audio.microphone.studio, studioSound),
          voice: flippedSwitch(audio.microphone.voice, vocalCleanup),
        }
      : audio?.microphone;
  const heard =
    audio && (audioVolume !== undefined || flipping)
      ? {
          ...audio,
          decibels: audioVolume ?? audio.decibels,
          microphone,
        }
      : null;
  const selection = layer ?? heard ?? resolved.selection;
  // The output choice flips at once, and the placement the editor carries
  // across arrives with its answer.
  const cameraOutput: ToolPanelSnapshot["cameraOutput"] =
    resolved.cameraOutput && bakeCamera !== undefined
      ? bakeCamera
        ? "combined"
        : "separate"
      : resolved.cameraOutput;
  // A colour being dragged in the system panel shows at once: the mirror
  // catches up an edit later, and the swatch must not blink back meanwhile.
  // The Animate and Sway switches, the aim and the words chosen are held the
  // same way, so none flicks back to the mirror's value between the edit and
  // the commit.
  const aimed =
    resolved.annotation && annotationAngle !== undefined
      ? { ...resolved.annotation, angle: annotationAngle }
      : resolved.annotation;
  const dialled =
    aimed?.rotation && annotationRotation
      ? { ...aimed, rotation: { ...aimed.rotation, ...annotationRotation } }
      : aimed;
  const switched =
    dialled && annotationAnimated !== undefined
      ? { ...dialled, animated: annotationAnimated }
      : dialled;
  const swayed =
    switched && annotationImageSway !== undefined
      ? { ...switched, sway: annotationImageSway }
      : switched;
  const chosen =
    swayed && annotationEmphasis
      ? { ...swayed, emphasis: annotationEmphasis }
      : swayed;
  const annotation =
    chosen && annotationStyle
      ? { ...chosen, style: { ...chosen.style, ...annotationStyle } }
      : chosen;
  return {
    ...resolved,
    annotation,
    cameraOutput,
    selection,
    ...(cropSize ? { crop: sized(resolved.crop, cropSize) } : {}),
    ...(frameSize ? { frame: sized(resolved.frame, frameSize) } : {}),
    ...(frameRadius !== undefined && resolved.frame
      ? { frame: { ...resolved.frame, radius: frameRadius } }
      : {}),
    // A scene chosen or reframed in the panel shows at once, the way the
    // output choice flips, until the editor answers with the scene it changed.
    ...(resolved.scene &&
    (scenePreset ||
      sceneFraming ||
      sceneRadius !== undefined ||
      sceneRotation ||
      sceneVariant)
      ? {
          scene: {
            ...resolved.scene,
            framing:
              resolved.scene.framing && sceneFraming
                ? { ...resolved.scene.framing, ...sceneFraming }
                : resolved.scene.framing,
            preset: scenePreset ?? resolved.scene.preset,
            radius:
              resolved.scene.radius === null
                ? null
                : (sceneRadius ?? resolved.scene.radius),
            rotation:
              resolved.scene.rotation && sceneRotation
                ? { ...resolved.scene.rotation, ...sceneRotation }
                : resolved.scene.rotation,
            variant:
              resolved.scene.variant && sceneVariant
                ? { ...resolved.scene.variant, ...sceneVariant }
                : resolved.scene.variant,
          },
        }
      : {}),
    ...(selectionOutput
      ? { selection: placedSelection(selection, selectionOutput) }
      : {}),
  };
}
