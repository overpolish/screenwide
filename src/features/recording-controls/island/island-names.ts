// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Camera,
  CameraOff,
  type LucideIcon,
  Mic,
  MicOff,
  Volume2,
  VolumeX,
} from "lucide-react";

import { t } from "../../../i18n/i18n";

import type { IslandInputKind, IslandMode } from "./island-types";

// The words and glyphs the island uses for its inputs, its modes and its
// clock.

export const inputName = (kind: IslandInputKind) =>
  kind === "camera"
    ? t("recording-controls-camera")
    : kind === "microphone"
      ? t("recording-controls-microphone")
      : t("recording-controls-system-audio");

export const modeName = (mode: IslandMode) =>
  ({
    none: t("recording-controls-island-mode-none"),
    region: t("recording-controls-mode-region"),
    screen: t("recording-controls-mode-screen"),
    window: t("recording-controls-mode-window"),
  })[mode];

/** The recording's clock: minutes and seconds, with the hours in front once
 * it passes an hour. The row grows for them as it grows for anything else. */
export const clock = (seconds: number) => {
  const whole = Math.max(0, Math.floor(seconds));
  const pad = (value: number) => String(value).padStart(2, "0");
  const hours = Math.floor(whole / 3_600);
  const minutes = Math.floor(whole / 60) % 60;
  return hours > 0
    ? `${String(hours)}:${pad(minutes)}:${pad(whole % 60)}`
    : `${String(minutes)}:${pad(whole % 60)}`;
};

/** Each input's glyph, on and off. */
export const INPUT_GLYPHS: Record<
  IslandInputKind,
  { off: LucideIcon; on: LucideIcon }
> = {
  camera: { off: CameraOff, on: Camera },
  microphone: { off: MicOff, on: Mic },
  systemAudio: { off: VolumeX, on: Volume2 },
};

/** The inputs, in the order the island shows them. */
export const INPUTS: IslandInputKind[] = [
  "microphone",
  "camera",
  "systemAudio",
];
