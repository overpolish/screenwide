// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { patchCaptions } from "../recording/captions/recording-captions";
import { EditorKind } from "../types";

import { draftedLayer } from "./tool-panel-draft-layer";
import { ToolPanelPatch } from "./tool-panel-patch";
import { ToolPanelMicrophone, ToolPanelSnapshot } from "./tool-panel-store";

export type ToolPanelDraft = {
  seq: number;
  values: ToolPanelPatch;
  workspace: EditorKind;
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

/** A width being typed does not pin the height a drag on the picture is still
 * moving. */
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
  // An action rather than a value: nothing of it is shown locally, and the
  // editor's answer arrives as the next published snapshot.
  const {
    annotationAngle,
    annotationAnimated,
    annotationEmphasis,
    annotationImage: _annotationImage,
    annotationImagePlay: _annotationImagePlay,
    annotationImageSway,
    annotationKind: _annotationKind,
    annotationLook: _annotationLook,
    annotationPin: _annotationPin,
    annotationRotation,
    annotationSceneReset: _annotationSceneReset,
    annotationStyle,
    annotationZoom,
    audioClip: _audioClip,
    autoVolume,
    autoZoomScenes: _autoZoomScenes,
    cameraEffects,
    captionDefaults: _captionDefaults,
    captions,
    chooseCameraBackground: _chooseCameraBackground,
    chooseSceneTemplate: _chooseSceneTemplate,
    clearAutoZooms: _clearAutoZooms,
    cropAspect,
    cutStripItem: _cutStripItem,
    deleteAnnotations: _deleteAnnotations,
    deleteChosenShortcuts: _deleteChosenShortcuts,
    deleteScene: _deleteScene,
    deselectStripItem: _deselectStripItem,
    downloadCameraBackground: _downloadCameraBackground,
    downloadCameraObjects: _downloadCameraObjects,
    downloadStudioDucking: _downloadStudioDucking,
    downloadStudioSound: _downloadStudioSound,
    frameRadius,
    frameSize,
    keyboardEffects,
    libraryImage: _libraryImage,
    lullPlay: _lullPlay,
    pickCameraObjects: _pickCameraObjects,
    recenterSelection: _recenterSelection,
    reduceNoise,
    removeAnnotationColor: _removeAnnotationColor,
    removeCameraObject: _removeCameraObject,
    removePreset: _removePreset,
    removeSceneTemplate: _removeSceneTemplate,
    resetCrop: _resetCrop,
    resetFrame: _resetFrame,
    resetPlacement: _resetPlacement,
    restoreShortcuts: _restoreShortcuts,
    reverseAnnotation: _reverseAnnotation,
    saveAnnotationColor: _saveAnnotationColor,
    savePreset: _savePreset,
    saveSceneTemplate: _saveSceneTemplate,
    sceneCursor,
    sceneFollowFace,
    sceneFraming,
    sceneFramingPane: _sceneFramingPane,
    sceneInset,
    scenePlacementReset: _scenePlacementReset,
    scenePreset,
    sceneRadius,
    sceneRecenter: _sceneRecenter,
    sceneRotation,
    sceneVariant,
    selectionDropShadow,
    selectionFollowFace,
    selectionInset,
    selectionRadius,
    selectionRotation,
    showCameraObject: _showCameraObject,
    shuffleAnnotation: _shuffleAnnotation,
    soundAddition: _soundAddition,
    studioDucking,
    studioSound,
    swapScenePanes: _swapScenePanes,
    trackEnabled: _trackEnabled,
    trackVolume: _trackVolume,
    transcriptRequest: _transcriptRequest,
    trimLullToBreath: _trimLullToBreath,
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
  const layer = draftedLayer(
    resolved.selection?.kind === "audio" ? null : resolved.selection,
    {
      cameraEffects,
      selectionDropShadow,
      selectionFollowFace,
      selectionInset,
      selectionRadius,
      selectionRotation,
    },
  );
  // The microphone switches flip at once, on the selected track and on the
  // Sound island's, which holds every track.
  const flipping =
    reduceNoise !== undefined ||
    vocalCleanup !== undefined ||
    autoVolume !== undefined ||
    studioSound !== undefined ||
    studioDucking !== undefined;
  const flipped = (microphone: ToolPanelMicrophone): ToolPanelMicrophone => ({
    ...microphone,
    autoVolume: flippedSwitch(microphone.autoVolume, autoVolume),
    ducking: microphone.ducking && {
      ...microphone.ducking,
      state: flippedSwitch(microphone.ducking.state, studioDucking),
    },
    noise: flippedSwitch(microphone.noise, reduceNoise),
    studio: flippedSwitch(microphone.studio, studioSound),
    voice: flippedSwitch(microphone.voice, vocalCleanup),
  });
  const audio =
    resolved.selection?.kind === "audio" ? resolved.selection : null;
  const heard =
    audio?.microphone && flipping
      ? { ...audio, microphone: flipped(audio.microphone) }
      : null;
  const selection = layer ?? heard ?? resolved.selection;
  const soundTracks =
    resolved.soundTracks && flipping
      ? resolved.soundTracks.map((track) =>
          track.microphone
            ? { ...track, microphone: flipped(track.microphone) }
            : track,
        )
      : resolved.soundTracks;
  // A colour being dragged in the system panel shows at once: the mirror
  // catches up an edit later, and the swatch must not blink back meanwhile.
  // The Animate and Sway switches, the aim, the zoom and the words chosen are
  // held the same way, so none flicks back to the mirror's value between the
  // edit and the commit.
  const turned =
    resolved.annotation && annotationAngle !== undefined
      ? { ...resolved.annotation, angle: annotationAngle }
      : resolved.annotation;
  const aimed =
    turned && annotationZoom !== undefined
      ? { ...turned, zoom: annotationZoom }
      : turned;
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
    selection,
    soundTracks,
    ...(cropAspect && resolved.crop
      ? { crop: { ...resolved.crop, aspect: cropAspect } }
      : {}),
    ...(frameSize ? { frame: sized(resolved.frame, frameSize) } : {}),
    ...(frameRadius !== undefined && resolved.frame
      ? { frame: { ...resolved.frame, radius: frameRadius } }
      : {}),
    // A scene chosen or reframed in the panel shows at once, until the editor
    // answers with the scene it changed.
    ...(resolved.scene &&
    (scenePreset ||
      sceneFraming ||
      sceneCursor !== undefined ||
      sceneFollowFace !== undefined ||
      sceneInset !== undefined ||
      sceneRadius !== undefined ||
      sceneRotation ||
      sceneVariant)
      ? {
          scene: {
            ...resolved.scene,
            cursor:
              sceneCursor === undefined ? resolved.scene.cursor : sceneCursor,
            followsFace: sceneFollowFace ?? resolved.scene.followsFace,
            framing:
              resolved.scene.framing && sceneFraming
                ? { ...resolved.scene.framing, ...sceneFraming }
                : resolved.scene.framing,
            panes:
              sceneInset !== undefined && resolved.scene.panes.screen
                ? {
                    ...resolved.scene.panes,
                    screen: {
                      ...resolved.scene.panes.screen,
                      inset: sceneInset,
                    },
                  }
                : resolved.scene.panes,
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
  };
}
