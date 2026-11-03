// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/** A moment placed while recording, as its kind was when it was placed. */
export type RecordingMoment = {
  /** `#rrggbb`. */
  color: string;
  /** Which moment it is, counting from zero in the order they were placed. */
  index: number;
  kindId: string;
  name: string;
  /** The voice note recorded with it, if the key was held for one. */
  note: RecordingMomentNote | null;
  /** On the recording's own timeline, before any edit. */
  sourceMs: number;
};

type RecordingMomentNote = {
  durationMs: number;
  /** What was said, as far as transcription has got; null before anything
   * is known of it. */
  transcript: NoteTranscript | null;
  /** Peak levels from 0 to 1. */
  waveform: number[];
};

/** Where a voice note's transcript is. `noModel` names the model that would
 * transcribe it, for the note to offer to download. */
export type NoteTranscript =
  | { model: string; status: "noModel" }
  | { status: "failed" }
  | { status: "queued" }
  | { status: "ready"; text: string }
  | { status: "transcribing" };

export const getRecordingMoments = (artifactId: number) =>
  invoke<RecordingMoment[]>("get_recording_moments", { artifactId });

/** Any note's transcript has moved on: queued, started, finished, failed,
 * or a model or setting changed what can be done. Notes are read again. */
const NOTES_CHANGED_EVENT = "transcription://notes-changed";

export const listenToNoteTranscripts = (onChange: () => void) =>
  listen(NOTES_CHANGED_EVENT, onChange);

/** The voice note of the `moment`th moment, as WAV bytes. */
export const getRecordingMomentNote = (artifactId: number, moment: number) =>
  invoke<ArrayBuffer>("get_recording_moment_note", { artifactId, moment });
