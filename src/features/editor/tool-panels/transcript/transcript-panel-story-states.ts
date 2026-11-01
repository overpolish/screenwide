// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RECORDING_SCENE_PRESETS } from "../../recording/scenes/recording-scene-labels";
import { SceneSpanTarget } from "../../recording/scenes/recording-scene-span";
import { SceneOffer } from "../scene/scene-offer";
import { ToolPanelTranscript, TranscriptScene } from "../tool-panel-transcript";

/** A screen and a camera drawn into it: every preset is on offer. */
export const storySceneOffer: SceneOffer = {
  canPlaceCamera: true,
  isCombined: true,
  presets: RECORDING_SCENE_PRESETS,
  templates: [],
};

/** `scenes` after a pick in the words' menu. Scenes stand in as their
 * timing alone, which is all the menu reads: a change moves its scene. */
export const scenesAfterPick = (
  scenes: TranscriptScene[],
  { endMs, startMs, ...target }: SceneSpanTarget,
): TranscriptScene[] =>
  (target.type === "change"
    ? scenes.map((scene) =>
        scene.id === target.id ? { ...scene, endMs, startMs } : scene,
      )
    : [
        ...scenes,
        { endMs, id: crypto.randomUUID(), needsCamera: false, startMs },
      ]
  ).sort((a, b) => a.startMs - b.startMs);

/** Remove silences before it has run: the microphone's own, which a story
 * has no recording to listen to. */
export const idleSilences: ToolPanelTranscript["silences"] = {
  count: 0,
  durationMs: 0,
  progress: null,
  status: "idle",
};
