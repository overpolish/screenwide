// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  CameraOverlaySettings,
  CursorEffectSettings,
  KeyboardEffectSettings,
  AudioTrackVolume,
} from "./types";

const finite = (value: number, fallback: number) =>
  Number.isFinite(value) ? value : fallback;

/**
 * Guard the overlay against a value that is not a number.
 *
 * Its placement is in the screen output's own pixels, so the canvas is what a
 * fallback has to be measured against: the shares below are the ones the
 * overlay used to be written in.
 */
export const normalizedCameraOverlay = (
  settings: CameraOverlaySettings,
  canvas: { height: number; width: number },
): CameraOverlaySettings => {
  const width = Math.max(1, canvas.width);
  const height = Math.max(1, canvas.height);
  return {
    cameraWidth: finite(settings.cameraWidth, width * 0.25),
    cameraX: finite(settings.cameraX, width * 0.85),
    cameraY: finite(settings.cameraY, height * 0.15),
    frameHeight: finite(settings.frameHeight, height * 0.25),
    frameWidth: finite(settings.frameWidth, width * 0.25),
    frameX: finite(settings.frameX, width * 0.72),
    frameY: finite(settings.frameY, height * 0.03),
    radiusPercent: finite(settings.radiusPercent, 8),
  };
};

export const normalizedCursorEffects = (
  settings: CursorEffectSettings,
): CursorEffectSettings => ({
  ...settings,
  sizePercent: finite(settings.sizePercent, 100),
});

export const normalizedKeyboardEffects = (
  settings: KeyboardEffectSettings,
): KeyboardEffectSettings => ({
  animation:
    settings.animation === "fade" || settings.animation === "none"
      ? settings.animation
      : "pop",
  appearance: settings.appearance === "dark" ? "dark" : "light",
  bake: settings.bake,
  positionXPercent: settings.positionXPercent,
  positionYPercent: settings.positionYPercent,
  sizePercent: finite(settings.sizePercent, 100),
});
export const normalizedAudioTrackVolumes = (volumes: AudioTrackVolume[]) =>
  volumes.map((volume) => ({
    ...volume,
    decibels: finite(volume.decibels, 0),
  }));
