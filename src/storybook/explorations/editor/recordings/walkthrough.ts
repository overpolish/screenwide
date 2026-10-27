// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ACTIONS, RECORDING_S, SEGMENTS } from "../../ideas/ideas-data";
import { MARK_S } from "../editor-data";
import { recordingOf } from "../recording";

// The 48-second walkthrough the ideas explorations share: a screen with the
// camera in its corner, two stretches where the screen changes by itself.

const markAt = (at: number) => ({ end: at + MARK_S, start: at });

export const WALKTHROUGH = recordingOf({
  actions: ACTIONS,
  duration: RECORDING_S,
  look: "screen",
  markTimes: { 1: markAt(16.2), 2: markAt(8.9), 3: markAt(43.1) },
  moments: [],
  picture: [
    { end: 29.2, start: 26.6, state: "moving" },
    { end: 45.4, start: 43.6, state: "moving" },
  ],
  recordedPreset: "pip",
  rest: "still",
  scenes: [
    { end: 17, id: "scene-name", preset: "zoom", start: 12.4, zoom: 2 },
    { end: 27, id: "scene-create", preset: "zoom", start: 24.3, zoom: 1.5 },
    { end: 44, id: "scene-share", preset: "zoom", start: 41.3, zoom: 2 },
  ],
  sounds: [],
  speech: SEGMENTS.flatMap((segment) =>
    segment.kind === "speech" ? [segment] : [],
  ),
});
