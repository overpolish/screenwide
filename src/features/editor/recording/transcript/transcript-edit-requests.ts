// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { moveRecordingTimelineFootage } from "../../timeline/editing/recording-timeline-footage-move";
import { restoreRecordingTimelineSilences } from "../../timeline/editing/recording-timeline-silences";

import {
  allowRecordingTranscriptWord,
  applyRecordingTranscriptCorrection,
  applyRecordingTranscriptCorrections,
  cutRecordingTranscriptRanges,
  restoreAllRecordingTranscript,
  restoreRecordingTranscriptRanges,
} from "./recording-timeline-transcript";

import type {
  TranscriptPlayed,
  TranscriptRequest,
} from "../../tool-panels/tool-panel-transcript";
import type { TranscriptWord } from "../../tool-panels/transcript/transcript-words";

/** The Transcript panel's asks that change the timeline edit. */
export type TranscriptEditRequest =
  | Extract<
      TranscriptRequest,
      {
        type:
          | "correct"
          | "correct-many"
          | "cut"
          | "move"
          | "not-profanity"
          | "restore";
      }
    >
  | { type: "restore-all" };

const EDIT_REQUESTS: ReadonlySet<TranscriptRequest["type"]> = new Set([
  "correct",
  "correct-many",
  "cut",
  "move",
  "not-profanity",
  "restore",
  "restore-all",
]);

export const isTranscriptEditRequest = (
  request: TranscriptRequest,
): request is TranscriptEditRequest => EDIT_REQUESTS.has(request.type);

/** `edit` with `request` made of a recording `sourceDurationMs` long, whose
 * transcript heard `words`. */
export function editForTranscriptRequest(
  edit: RecordingTimelineEdit,
  request: TranscriptEditRequest,
  {
    sourceDurationMs,
    words,
  }: { sourceDurationMs: number; words: readonly TranscriptWord[] },
): RecordingTimelineEdit {
  switch (request.type) {
    case "cut":
      return cutRecordingTranscriptRanges(
        edit,
        request.ranges,
        sourceDurationMs,
      );
    case "restore":
      return restoreRecordingTranscriptRanges(
        edit,
        request.ranges,
        sourceDurationMs,
      );
    case "restore-all":
      return restoreRecordingTimelineSilences(
        restoreAllRecordingTranscript(edit),
      );
    case "not-profanity":
      return allowRecordingTranscriptWord(edit, request.text);
    case "correct":
      return applyRecordingTranscriptCorrection(edit, request.correction, {
        sourceDurationMs,
        words,
      });
    case "correct-many":
      return applyRecordingTranscriptCorrections(edit, request.corrections, {
        sourceDurationMs,
        words,
      });
    case "move":
      return moveRecordingTimelineFootage(edit, request.move, sourceDurationMs);
  }
}

/** The stretches of a recording `sourceDurationMs` long `edit` plays, in
 * the order it plays them, in milliseconds, each at its own speed. */
export const playedRecordingStretches = (
  edit: RecordingTimelineEdit,
  sourceDurationMs: number,
): TranscriptPlayed[] =>
  edit.segments.map((segment) => ({
    endMs: segment.sourceEnd * sourceDurationMs,
    rate: segment.playbackRate ?? 1,
    startMs: segment.sourceStart * sourceDurationMs,
  }));
