// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MicrophoneTools } from "../../recording/microphone/use-microphone-tools";
import { ToolPanelHandlers } from "../tool-panel-handlers";

import { EditorSelectionTarget } from "./placed-selection-target";

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
  | "onSelectionFollowFaceChange"
  | "onSelectionInsetChange"
  | "onSelectionPlacementReset"
  | "onSelectionRadiusChange"
  | "onSelectionRecenter"
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
  onSelectionFollowFaceChange: (follow) => {
    target?.applyFollowFace?.(follow);
  },
  // The pad is the layer's own frame grown past its picture, so it travels
  // with the layer's output the way its size and position do.
  onSelectionInsetChange: (inset) => {
    target?.applyInset(inset);
  },
  onSelectionPlacementReset: () => {
    target?.resetPlacement();
  },
  onSelectionRadiusChange: (radius) => {
    target?.applyRadius(Math.min(50, Math.max(0, radius)));
  },
  onSelectionRecenter: () => {
    target?.recenter();
  },
  onSelectionRotationChange: (rotation) => {
    target?.applyRotation?.(rotation);
  },
});

/** The speech tools, which are the recording's one microphone's whichever
 * track is selected. */
export const microphonePanelHandlers = (
  microphone: MicrophoneTools | null,
): Pick<
  ToolPanelHandlers,
  | "onAutoVolumeChange"
  | "onReduceNoiseChange"
  | "onStudioDuckingChange"
  | "onStudioDuckingDownload"
  | "onStudioSoundChange"
  | "onStudioSoundDownload"
  | "onVocalCleanupChange"
> => ({
  onAutoVolumeChange: (enabled) => {
    microphone?.changeAutoVolume(enabled);
  },
  onReduceNoiseChange: (enabled) => {
    microphone?.reduceNoise(enabled);
  },
  onStudioDuckingChange: (enabled) => {
    microphone?.changeStudioDucking(enabled);
  },
  onStudioDuckingDownload: () => {
    microphone?.downloadStudioDucking();
  },
  onStudioSoundChange: (enabled) => {
    microphone?.changeStudioSound(enabled);
  },
  onStudioSoundDownload: () => {
    microphone?.downloadStudioSound();
  },
  onVocalCleanupChange: (enabled) => {
    microphone?.cleanUpVoice(enabled);
  },
});
