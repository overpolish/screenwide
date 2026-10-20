// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AppWindow,
  Mic,
  MicOff,
  Monitor,
  MonitorOff,
  SquareDashed,
  Video,
  VideoOff,
  Volume2,
  VolumeX,
} from "lucide-react";

// Camera only and audio only are not targets of their own: with No screen,
// the inputs alone decide, so a camera means a camera recording and no
// camera means sound only. The camera then has one control, its tile.
export type TargetId = "area" | "display" | "none" | "window";

export const TARGETS: {
  icon: typeof Monitor;
  id: TargetId;
  label: string;
}[] = [
  { icon: Monitor, id: "display", label: "Display" },
  { icon: AppWindow, id: "window", label: "Window" },
  { icon: SquareDashed, id: "area", label: "Area" },
  { icon: MonitorOff, id: "none", label: "No screen" },
];

export const DISPLAY = { id: "studio", name: "Studio Display" };
export const DISPLAYS = [DISPLAY, { id: "built-in", name: "Built-in Display" }];
export const WINDOW = {
  app: "Safari",
  name: "Screenwide docs",
  size: "1440 × 900",
};
export const AREA = { size: "1280 × 720" };

export type InputId = "camera" | "microphone" | "sound";

export type InputSource = {
  device: string;
  icon: typeof Mic;
  id: InputId;
  label: string;
  offIcon: typeof Mic;
  /** Live level in decibels; the camera has none. */
  decibels?: number;
};

export const INPUTS: InputSource[] = [
  {
    device: "FaceTime HD Camera",
    icon: Video,
    id: "camera",
    label: "Camera",
    offIcon: VideoOff,
  },
  {
    decibels: -14,
    device: "MacBook Pro Microphone",
    icon: Mic,
    id: "microphone",
    label: "Microphone",
    offIcon: MicOff,
  },
  {
    decibels: -24,
    device: "All audio",
    icon: Volume2,
    id: "sound",
    label: "Sound",
    offIcon: VolumeX,
  },
];

/** What a person usually records with: microphone and sound, no camera. */
export const DEFAULT_INPUTS: Record<InputId, boolean> = {
  camera: false,
  microphone: true,
  sound: true,
};
