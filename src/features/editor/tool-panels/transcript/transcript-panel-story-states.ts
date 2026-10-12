// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RECORDING_SCENE_PRESETS } from "../../recording/scenes/recording-scene-labels";
import { SceneSpanTarget } from "../../recording/scenes/recording-scene-span";
import { SceneOffer } from "../scene/scene-offer";
import { ToolPanelTranscript, TranscriptScene } from "../tool-panel-transcript";

import { sampleTranscriptDurationMs } from "./transcript-panel-fixtures";

import type { TranscriptPanelViewProps } from "./transcript-panel-view";

export const ignore = () => undefined;

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

/** A panel that asks for nothing, for the states before there are words. */
export const still = (
  transcript: ToolPanelTranscript,
): TranscriptPanelViewProps => ({
  isLocked: false,
  language: "en",
  onCancel: ignore,
  onCorrect: ignore,
  onCorrectMany: ignore,
  onCut: ignore,
  onDownload: ignore,
  onHighlight: ignore,
  onMove: ignore,
  onNotProfanity: ignore,
  onRedo: ignore,
  onRemoveSilences: ignore,
  onRestore: ignore,
  onRestoreAll: ignore,
  onScene: ignore,
  onSeek: ignore,
  onTogglePlayback: ignore,
  onTranscribe: ignore,
  onUndo: ignore,
  plannedSilences: [],
  playheadMs: 0,
  sceneOffer: null,
  transcript,
  words: null,
});

/** The transcript as it stands before there are words. */
export const before = (
  model: ToolPanelTranscript["model"],
  status: ToolPanelTranscript["status"] = "none",
  progress = 0,
): ToolPanelTranscript => ({
  allowedWords: [],
  artifactId: 1,
  canRestore: false,
  corrections: [],
  durationMs: sampleTranscriptDurationMs,
  error: null,
  model,
  played: [],
  progress,
  removed: [],
  revision: 0,
  scenes: [],
  silences: idleSilences,
  status,
});
