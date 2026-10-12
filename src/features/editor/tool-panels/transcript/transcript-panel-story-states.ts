// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ToolPanelTranscript } from "../tool-panel-transcript";

import { sampleTranscriptDurationMs } from "./transcript-panel-fixtures";

import type { TranscriptPanelViewProps } from "./transcript-panel-view";

export const ignore = () => undefined;

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
  onNotProfanity: ignore,
  onRedo: ignore,
  onRemoveSilences: ignore,
  onRestore: ignore,
  onRestoreAll: ignore,
  onSeek: ignore,
  onTogglePlayback: ignore,
  onTranscribe: ignore,
  onUndo: ignore,
  plannedSilences: [],
  playheadMs: 0,
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
  progress,
  removed: [],
  revision: 0,
  silences: idleSilences,
  status,
});
