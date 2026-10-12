// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  cutSegments,
  mergeAddedSegments,
  removedParts,
  sharesOf,
  stillRemoved,
} from "../../timeline/editing/recording-timeline-cut-ranges";
import {
  RecordingTimelineEdit,
  RecordingTimelineSegment,
} from "../../timeline/editing/recording-timeline-edit";
import {
  earliestSourceIndex,
  sourcePredecessorIndex,
} from "../../timeline/editing/recording-timeline-source-order";
import { inheritRecordingKeyboardSegmentEdits } from "../../timeline/keyboard/recording-keyboard-timeline-edit";

import { correctionsWith } from "./transcript-corrections";

import type {
  TranscriptCorrectionSpan,
  TranscriptHeardSpan,
} from "./transcript-corrections";
import type { RecordingTranscriptCorrection } from "../../../../bindings/RecordingTranscriptCorrection";
import type { RecordingTranscriptCut } from "../../../../bindings/RecordingTranscriptCut";
import type { SourceCut } from "../../timeline/editing/recording-timeline-cut-ranges";
import type {
  TranscriptCorrection,
  TranscriptRange,
} from "../../tool-panels/tool-panel-transcript";
import type { TranscriptWord } from "../../tool-panels/transcript/transcript-words";

/** The recorded transcript cuts, cut down to what is still cut out. */
const stillRemovedTranscript = (
  edit: RecordingTimelineEdit,
): RecordingTranscriptCut[] =>
  stillRemoved(edit.segments, edit.transcriptCuts ?? []);

/**
 * Cuts `ranges`, in milliseconds of a recording `sourceDurationMs` long, out
 * of whatever of them the edit still keeps, and records what the Transcript
 * panel took for Restore all. One call is one step of the edit history.
 */
export function cutRecordingTranscriptRanges(
  edit: RecordingTimelineEdit,
  ranges: readonly TranscriptRange[],
  sourceDurationMs: number,
): RecordingTimelineEdit {
  if (sourceDurationMs <= 0) return edit;
  const shares = sharesOf(ranges, sourceDurationMs);
  if (shares.length === 0) return edit;
  const { descendantIds, nextSegmentId, segments, taken } = cutSegments(
    edit,
    shares,
  );
  if (taken.length === 0 || segments.length === 0) return edit;
  return inheritRecordingKeyboardSegmentEdits(
    edit,
    {
      ...edit,
      nextSegmentId,
      segments,
      transcriptCuts: [...stillRemovedTranscript(edit), ...taken].sort(
        (a, b) => a.sourceStart - b.sourceStart,
      ),
    },
    descendantIds,
  );
}

/**
 * Brings back whatever `ranges` leave out, in milliseconds: any cut at all,
 * from Remove silences, the Transcript panel, or a trim of your own, each at
 * the rate it played at. The silence and transcript records shrink with
 * what is no longer cut. One call is one step of the edit history.
 */
export function restoreRecordingTranscriptRanges(
  edit: RecordingTimelineEdit,
  ranges: readonly TranscriptRange[],
  sourceDurationMs: number,
): RecordingTimelineEdit {
  if (sourceDurationMs <= 0) return edit;
  const shares = sharesOf(ranges, sourceDurationMs);
  if (shares.length === 0) return edit;
  const silences = stillRemoved(edit.segments, edit.silenceCuts ?? []);
  const transcript = stillRemoved(edit.segments, edit.transcriptCuts ?? []);
  // The rate a gap plays at once restored: a recorded cut's own rate, or
  // the segment beside it for a cut of your own.
  const rateFor = (gap: { end: number; start: number }): number => {
    const silence = silences.find(
      (cut) => cut.sourceStart < gap.end && gap.start < cut.sourceEnd,
    );
    if (silence) return silence.playbackRate;
    const corrected = transcript.find(
      (cut) => cut.sourceStart < gap.end && gap.start < cut.sourceEnd,
    );
    if (corrected) return corrected.playbackRate;
    // An edit always keeps a segment, so one of these is always found.
    const predecessor = sourcePredecessorIndex(edit.segments, gap.start);
    const beside =
      edit.segments[
        predecessor >= 0 ? predecessor : earliestSourceIndex(edit.segments)
      ];
    return beside.playbackRate ?? 1;
  };
  const gaps: SourceCut[] = [];
  for (const range of shares) {
    for (const part of removedParts(edit.segments, range)) {
      gaps.push({
        playbackRate: rateFor(part),
        sourceEnd: part.end,
        sourceStart: part.start,
      });
    }
  }
  if (gaps.length === 0) return edit;
  let nextSegmentId = edit.nextSegmentId;
  const added = gaps.map((gap): RecordingTimelineSegment => ({
    id: nextSegmentId++,
    sourceEnd: gap.sourceEnd,
    sourceStart: gap.sourceStart,
    ...(gap.playbackRate === 1 ? {} : { playbackRate: gap.playbackRate }),
  }));
  const segments = mergeAddedSegments(edit, added);
  const silenceCuts = stillRemoved(segments, edit.silenceCuts ?? []);
  const transcriptCuts = stillRemoved(segments, edit.transcriptCuts ?? []);
  const { silenceCuts: _silence, transcriptCuts: _transcript, ...rest } = edit;
  return {
    ...rest,
    nextSegmentId,
    segments,
    ...(silenceCuts.length > 0 ? { silenceCuts } : {}),
    ...(transcriptCuts.length > 0 ? { transcriptCuts } : {}),
  };
}

/**
 * Brings back everything the Transcript panel cut that is still cut, each at
 * the rate it played at. One call is one step of the edit history.
 */
export function restoreAllRecordingTranscript(
  edit: RecordingTimelineEdit,
): RecordingTimelineEdit {
  const removed = stillRemovedTranscript(edit);
  if (removed.length === 0) {
    if (!edit.transcriptCuts?.length) return edit;
    const { transcriptCuts: _, ...rest } = edit;
    return rest;
  }
  let nextSegmentId = edit.nextSegmentId;
  const added = removed.map((cut): RecordingTimelineSegment => ({
    id: nextSegmentId++,
    sourceEnd: cut.sourceEnd,
    sourceStart: cut.sourceStart,
    ...(cut.playbackRate === 1 ? {} : { playbackRate: cut.playbackRate }),
  }));
  const segments = mergeAddedSegments(edit, added);
  const { transcriptCuts: _, ...rest } = edit;
  return { ...rest, nextSegmentId, segments };
}

/**
 * `edit` once the words `change` covers are corrected to its text, applying
 * the same rule the panel shows: it replaces every correction it overlaps,
 * and text that is what was heard takes the correction away.
 */
export function applyRecordingTranscriptCorrection(
  edit: RecordingTimelineEdit,
  change: TranscriptCorrection,
  source: {
    sourceDurationMs: number;
    words: readonly TranscriptWord[];
  },
): RecordingTimelineEdit {
  return applyRecordingTranscriptCorrections(edit, [change], source);
}

/**
 * `edit` once every change in `changes` is applied in turn, each by the
 * same rule {@link applyRecordingTranscriptCorrection} does: one undo step
 * for Replace all's several corrections at once.
 */
export function applyRecordingTranscriptCorrections(
  edit: RecordingTimelineEdit,
  changes: readonly TranscriptCorrection[],
  source: {
    sourceDurationMs: number;
    words: readonly TranscriptWord[];
  },
): RecordingTimelineEdit {
  const { sourceDurationMs, words } = source;
  if (sourceDurationMs <= 0 || changes.length === 0) return edit;
  const heard = words.map((word): TranscriptHeardSpan => ({
    end: word.endMs / sourceDurationMs,
    start: word.startMs / sourceDurationMs,
    text: word.text,
  }));
  let corrections = (edit.transcriptCorrections ?? []).map(
    (correction): TranscriptCorrectionSpan => ({
      end: correction.sourceEnd,
      start: correction.sourceStart,
      text: correction.text,
    }),
  );
  for (const change of changes) {
    corrections = correctionsWith(heard, corrections, {
      end: change.endMs / sourceDurationMs,
      start: change.startMs / sourceDurationMs,
      text: change.text,
    });
  }
  return {
    ...edit,
    transcriptCorrections: corrections.map(
      (correction): RecordingTranscriptCorrection => ({
        sourceEnd: correction.end,
        sourceStart: correction.start,
        text: correction.text,
      }),
    ),
  };
}

/** Every stretch the edit leaves out, in milliseconds, sorted and never
 * overlapping: the words inside show struck through. */
export function recordingTranscriptRemovedMs(
  edit: RecordingTimelineEdit,
  sourceDurationMs: number,
): TranscriptRange[] {
  return removedParts(edit.segments, { end: 1, start: 0 }).map((part) => ({
    endMs: Math.round(part.end * sourceDurationMs),
    startMs: Math.round(part.start * sourceDurationMs),
  }));
}

/** The corrections in the edit, in milliseconds, so the panel can compare
 * them against the words it is showing. */
export function recordingTranscriptCorrectionsMs(
  edit: RecordingTimelineEdit,
  sourceDurationMs: number,
): TranscriptCorrection[] {
  return (edit.transcriptCorrections ?? []).map((correction) => ({
    endMs: Math.round(correction.sourceEnd * sourceDurationMs),
    startMs: Math.round(correction.sourceStart * sourceDurationMs),
    text: correction.text,
  }));
}

/** Whether anything the Transcript panel cut is still cut. */
export function recordingTranscriptCanRestore(
  edit: RecordingTimelineEdit,
): boolean {
  return stillRemovedTranscript(edit).length > 0;
}

/** What the Transcript panel was told is not profanity, as the folded text
 * each matched. */
export const recordingTranscriptAllowedWords = (edit: RecordingTimelineEdit) =>
  edit.transcriptAllowedWords ?? [];

/** The edit with `folded` marked "Not profanity", or the edit itself where
 * it already was. */
export function allowRecordingTranscriptWord(
  edit: RecordingTimelineEdit,
  folded: string,
): RecordingTimelineEdit {
  const allowed = recordingTranscriptAllowedWords(edit);
  if (folded === "" || allowed.includes(folded)) return edit;
  return { ...edit, transcriptAllowedWords: [...allowed, folded].sort() };
}
