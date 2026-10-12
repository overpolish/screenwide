// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { RecordingTranscript } from "../../../../bindings/RecordingTranscript";
import type { TranscriptProgress } from "../../../../bindings/TranscriptProgress";

export type { RecordingTranscript, TranscriptProgress };

/** How far transcribing a recording has got, as the editor is told while it
 * works; `src-tauri/src/editor/speech/transcript.rs`. */
export const listenToTranscriptProgress = (
  onProgress: (progress: TranscriptProgress) => void,
) =>
  listen<TranscriptProgress>("editor://transcript-progress", (event) => {
    onProgress(event.payload);
  });

/** Where the playhead stands, in source milliseconds, sent by the editor to
 * the Transcript panel while it is open. */
export const TRANSCRIPT_PLAYHEAD_EVENT = "editor://transcript-playhead";

/** The playhead's source position, as the editor sends it to the panel. */
export type TranscriptPlayhead = { playheadMs: number };

/** Transcribes the recording open in the editor afresh, word by word, from
 * `src-tauri/src/editor/speech/transcript.rs`. Refused while it is already
 * transcribing; cancel it with {@link cancelRecordingTranscription}. */
export const transcribeRecording = (artifactId: number) =>
  invoke<RecordingTranscript>("transcribe_recording", { artifactId });

/** Asks to stop the transcription of the recording under way, if there is
 * one. */
export const cancelRecordingTranscription = (artifactId: number) =>
  invoke<null>("cancel_recording_transcription", { artifactId });

/** The transcript already made for the recording open in the editor, if any. */
export const getRecordingTranscript = (artifactId: number) =>
  invoke<RecordingTranscript | null>("get_recording_transcript", {
    artifactId,
  });
