// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ToolPanelModel, ToolPanelSilences } from "./tool-panel-selection";

/** A stretch of the recording, in milliseconds from the start of the source. */
export type TranscriptRange = { endMs: number; startMs: number };

/** Text of your own for the words a stretch covers: always whole words, from
 * the first one's start to the last one's end. The recording is unchanged. */
export type TranscriptCorrection = TranscriptRange & { text: string };

/**
 * What the Transcript panel shows of the recording's transcript. The words
 * themselves travel apart, through `transcript-channel`: a long recording has
 * thousands, too many to republish with every change of a setting.
 */
export type ToolPanelTranscript = {
  /** What was marked "Not profanity", as the folded text each matched, so
   * it goes unmarked throughout the recording. */
  allowedWords: string[];
  /** The recording whose words these are. */
  artifactId: number;
  /** Whether anything the panel cut, or Remove silences cut, is still cut:
   * Restore all brings back both. */
  canRestore: boolean;
  /** Words corrected by hand, sorted by start and never overlapping. */
  corrections: TranscriptCorrection[];
  /** How long the recording is, in milliseconds, so the silence after the
   * last word can be shown and cut, and a cut of the last words knows where
   * the recording ends. */
  durationMs: number;
  /** What went wrong transcribing, or null while nothing has. */
  error: string | null;
  /** The Transcripts model and how far it is on this computer; `null` until
   * the app has said. */
  model: ToolPanelModel | null;
  /** How far transcribing has got, 0 to 1, while `transcribing`. */
  progress: number;
  /** What the edit leaves out of the source, by any cut, sorted by start and
   * never overlapping: the words inside show struck through. */
  removed: TranscriptRange[];
  /** Bumps each time the transcript's words change, so the panel refetches
   * them from the backend. */
  revision: number;
  /** Remove silences, the microphone's own and the same state its Select
   * panel shows; `null` until the microphone's tools are ready. */
  silences: ToolPanelSilences | null;
  status: "none" | "ready" | "transcribing";
};

/** What the Transcript panel asks of the editor. Cuts and restores name
 * stretches of the source, each one run of words chosen in the transcript.
 * A correction replaces every correction it overlaps, and one whose text is
 * what was heard takes the correction away; `correctionsWith` is the rule.
 * `correct-many` applies several corrections as one undo step, for Replace
 * all. `restore-all` brings back the panel's cuts and Remove silences' in
 * one undo step. `not-profanity` stops `text`, folded, being marked as
 * profanity anywhere in the recording, as a step of the edit history. */
export type TranscriptRequest =
  | { correction: TranscriptCorrection; type: "correct" }
  | { corrections: TranscriptCorrection[]; type: "correct-many" }
  | { ranges: TranscriptRange[]; type: "cut" | "restore" }
  | { ms: number; type: "seek" }
  | { range: TranscriptRange | null; type: "highlight" }
  | { text: string; type: "not-profanity" }
  | {
      type:
        | "cancel"
        | "download"
        | "remove-silences"
        | "restore-all"
        | "transcribe";
    };
