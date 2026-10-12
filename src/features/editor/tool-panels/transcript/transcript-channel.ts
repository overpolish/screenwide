// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";

import type { TranscriptRange } from "../tool-panel-transcript";
import type { TranscriptWord } from "./transcript-words";

/**
 * The transcript's words and the playhead, as the Transcript panel's window
 * holds them. Kept apart from the tool panel mirror, which is republished
 * with every change of a setting: a long recording's words would travel with
 * each one, and the playhead moves on every frame while the recording plays.
 */
export const useTranscriptChannel = create<{
  /** The language the transcriber heard, as its code, for the filler and
   * profanity lists; null until the transcript has been read. */
  language: string | null;
  /** What Remove silences would cut, read with the words so hovering Remove
   * can show it at once; empty until read. */
  plannedSilences: TranscriptRange[];
  /** Where the playhead is, in source milliseconds. */
  playheadMs: number;
  /** Null until the transcript has been read. */
  words: TranscriptWord[] | null;
}>()(() => ({
  language: null,
  plannedSilences: [],
  playheadMs: 0,
  words: null,
}));
