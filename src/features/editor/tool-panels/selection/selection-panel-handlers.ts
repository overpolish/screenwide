// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ToolPanelHandlers } from "../tool-panel-handlers";

import { EditorSelectionTarget } from "./placed-selection-target";
import { EditorAudioSelectionTarget } from "./selection-target";

/**
 * The selection panel's asks, answered against whatever is selected now.
 *
 * Each goes back out through the editor's own handler for it, so a number
 * typed in the panel takes the identical path through the editor's state as
 * the drag that would have produced it.
 */
export const selectionPanelHandlers = (
  target: EditorSelectionTarget | null,
): Pick<
  ToolPanelHandlers,
  | "onCameraBackgroundChoose"
  | "onCameraBackgroundDownload"
  | "onCameraEffectsChange"
  | "onCameraObjectRemove"
  | "onCameraObjectShow"
  | "onCameraObjectsDownload"
  | "onCameraObjectsPick"
  | "onSelectionDropShadowChange"
  | "onSelectionInsetChange"
  | "onSelectionPlacementChange"
  | "onSelectionRadiusChange"
  | "onSelectionRecenter"
  | "onSelectionReset"
  | "onSelectionRotationChange"
> => ({
  onCameraBackgroundChoose: () => {
    target?.cameraEffects?.chooseBackground();
  },
  onCameraBackgroundDownload: () => {
    target?.cameraEffects?.download();
  },
  onCameraEffectsChange: (change) => {
    target?.cameraEffects?.change(change);
  },
  onCameraObjectRemove: (id) => {
    target?.cameraEffects?.objects?.remove(id);
  },
  onCameraObjectShow: (id) => {
    target?.cameraEffects?.objects?.show(id);
  },
  onCameraObjectsDownload: () => {
    target?.cameraEffects?.objects?.download();
  },
  onCameraObjectsPick: (isPicking) => {
    target?.cameraEffects?.objects?.setPicking(isPicking);
  },
  onSelectionDropShadowChange: (dropShadow) => {
    target?.applyDropShadow(dropShadow);
  },
  // The pad is the layer's own frame grown past its picture, so it travels
  // with the layer's output the way its size and position do.
  onSelectionInsetChange: (inset) => {
    target?.applyInset(inset);
  },
  onSelectionPlacementChange: (placement) => {
    target?.applyPlacement(placement);
  },
  onSelectionRadiusChange: (radius) => {
    target?.applyRadius(Math.min(50, Math.max(0, radius)));
  },
  onSelectionRecenter: () => {
    target?.recenter();
  },
  onSelectionReset: () => {
    target?.reset();
  },
  onSelectionRotationChange: (rotation) => {
    target?.applyRotation?.(rotation);
  },
});

/** The volume and speech tools the audio selection offers, answered against
 * the selected track. */
export const audioPanelHandlers = (
  target: EditorAudioSelectionTarget | null,
): Pick<
  ToolPanelHandlers,
  | "onAudioVolumeChange"
  | "onAutoVolumeChange"
  | "onReduceNoiseChange"
  | "onSilencesRemove"
  | "onSilencesRestore"
  | "onStudioDuckingChange"
  | "onStudioDuckingDownload"
  | "onStudioSoundChange"
  | "onStudioSoundDownload"
  | "onVocalCleanupChange"
> => ({
  onAudioVolumeChange: (decibels) => {
    target?.applyVolume(decibels);
  },
  onAutoVolumeChange: (enabled) => {
    target?.microphone?.changeAutoVolume(enabled);
  },
  onReduceNoiseChange: (enabled) => {
    target?.microphone?.reduceNoise(enabled);
  },
  onSilencesRemove: () => {
    target?.microphone?.removeSilences();
  },
  onSilencesRestore: () => {
    target?.microphone?.restoreSilences();
  },
  onStudioDuckingChange: (enabled) => {
    target?.microphone?.changeStudioDucking(enabled);
  },
  onStudioDuckingDownload: () => {
    target?.microphone?.downloadStudioDucking();
  },
  onStudioSoundChange: (enabled) => {
    target?.microphone?.changeStudioSound(enabled);
  },
  onStudioSoundDownload: () => {
    target?.microphone?.downloadStudioSound();
  },
  onVocalCleanupChange: (enabled) => {
    target?.microphone?.cleanUpVoice(enabled);
  },
});
