// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// What the editor offers, whatever the recording (`recording.ts`): the
// scene arrangements, zoom levels, backdrops and how long annotations show.

/** How a scene arranges the screen and the camera
 * (`src-tauri/crates/scenes`): a zoom reframes the screen as recorded. */
export type ScenePreset =
  "camera" | "pip" | "screen" | "side" | "stacked" | "zoom";

export const SCENE_PRESETS: { id: ScenePreset; label: string }[] = [
  { id: "zoom", label: "Zoom" },
  { id: "side", label: "Side by side" },
  { id: "pip", label: "Camera in corner" },
  { id: "stacked", label: "Screen above camera" },
  { id: "camera", label: "Camera only" },
  { id: "screen", label: "Screen only" },
];

export const presetLabel = (id: ScenePreset) =>
  SCENE_PRESETS.find((preset) => preset.id === id)?.label ?? id;

/** A scene over a stretch of the recording, in recorded seconds, so a cut
 * or a squeeze never moves where it sits. */
export type SceneClip = {
  end: number;
  id: string;
  preset: ScenePreset;
  start: number;
  /** How far a zoom goes in. */
  zoom: number;
};

export const ZOOMS = [1.5, 2, 2.5];

/** How long a new annotation shows, in seconds, from what it is about. */
export const MARK_S = 4;

export type Backdrop = "dark" | "none" | "warm";

export const BACKDROPS: { background: string; id: Backdrop; label: string }[] =
  [
    { background: "transparent", id: "none", label: "None" },
    {
      background: "linear-gradient(135deg, #f6d365, #fda085)",
      id: "warm",
      label: "Warm",
    },
    {
      background: "linear-gradient(135deg, #2b2d42, #111)",
      id: "dark",
      label: "Dark",
    },
  ];

export const backdropOf = (id: Backdrop) =>
  BACKDROPS.find((item) => item.id === id)?.background ?? "transparent";

/** The scene over a moment, if there is one. */
export const sceneAt = (scenes: SceneClip[], seconds: number) =>
  scenes.find(({ end, start }) => seconds >= start && seconds < end);
