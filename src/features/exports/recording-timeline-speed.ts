// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { inheritRecordingKeyboardSegmentEdits } from "./recording-keyboard-timeline-edit";
import {
  layoutRecordingTimelineSegments,
  RecordingTimelineEdit,
  RecordingTimelineSegment,
  recordingTimelineRetainedDuration,
} from "./recording-timeline-edit";

const RANGE_EPSILON = 1e-9;

const normalizedRange = (start: number, end: number) => {
  if (!Number.isFinite(start) || !Number.isFinite(end)) return null;
  const normalizedStart = Math.max(0, Math.min(1, Math.min(start, end)));
  const normalizedEnd = Math.max(0, Math.min(1, Math.max(start, end)));
  return normalizedEnd - normalizedStart > RANGE_EPSILON
    ? { end: normalizedEnd, start: normalizedStart }
    : null;
};

/** Applies one rate to retained content selected in magnetic output time. */
export function setRecordingTimelineRangePlaybackRate(
  edit: RecordingTimelineEdit,
  {
    outputEnd,
    outputStart,
    playbackRate,
  }: { outputEnd: number; outputStart: number; playbackRate: number },
): RecordingTimelineEdit {
  const range = normalizedRange(outputStart, outputEnd);
  if (
    !range ||
    !Number.isFinite(playbackRate) ||
    playbackRate < 0.25 ||
    playbackRate > 4
  )
    return edit;
  const retainedDuration = recordingTimelineRetainedDuration(edit);
  if (retainedDuration <= 0) return edit;
  const selectionStart = range.start * retainedDuration;
  const selectionEnd = range.end * retainedDuration;
  let retainedStart = 0;
  let nextSegmentId = edit.nextSegmentId;
  const descendantIds = new Map<number, number[]>();
  const segments = edit.segments.flatMap((segment) => {
    const currentRate = segment.playbackRate ?? 1;
    const duration = (segment.sourceEnd - segment.sourceStart) / currentRate;
    const retainedEnd = retainedStart + duration;
    const overlapStart = Math.max(selectionStart, retainedStart);
    const overlapEnd = Math.min(selectionEnd, retainedEnd);
    const segmentRetainedStart = retainedStart;
    retainedStart = retainedEnd;
    if (
      overlapEnd - overlapStart <= RANGE_EPSILON ||
      currentRate === playbackRate
    )
      return [segment];

    const selectedSourceStart =
      segment.sourceStart + (overlapStart - segmentRetainedStart) * currentRate;
    const selectedSourceEnd =
      segment.sourceStart + (overlapEnd - segmentRetainedStart) * currentRate;
    const pieces: RecordingTimelineSegment[] = [];
    const pushPiece = (
      sourceStart: number,
      sourceEnd: number,
      rate: number | undefined,
    ) => {
      if (sourceEnd - sourceStart <= RANGE_EPSILON) return;
      const piece: RecordingTimelineSegment = {
        ...segment,
        id: pieces.length === 0 ? segment.id : nextSegmentId++,
        sourceEnd,
        sourceStart,
      };
      if (rate === undefined) delete piece.playbackRate;
      else piece.playbackRate = rate;
      pieces.push(piece);
    };
    pushPiece(segment.sourceStart, selectedSourceStart, segment.playbackRate);
    pushPiece(selectedSourceStart, selectedSourceEnd, playbackRate);
    pushPiece(selectedSourceEnd, segment.sourceEnd, segment.playbackRate);
    const descendants = pieces
      .filter((piece) => piece.id !== segment.id)
      .map((piece) => piece.id);
    if (descendants.length > 0) descendantIds.set(segment.id, descendants);
    return pieces;
  });
  const changed =
    segments.length !== edit.segments.length ||
    segments.some((segment, index) => segment !== edit.segments[index]);
  if (!changed) return edit;
  return inheritRecordingKeyboardSegmentEdits(
    edit,
    {
      ...edit,
      nextSegmentId,
      segments,
    },
    descendantIds,
  );
}

/** Returns the common rate across a selected output range, if it has one. */
export function recordingTimelineRangePlaybackRate(
  edit: RecordingTimelineEdit,
  outputStart: number,
  outputEnd: number,
) {
  const range = normalizedRange(outputStart, outputEnd);
  if (!range) return undefined;
  let playbackRate: number | undefined;
  for (const segment of layoutRecordingTimelineSegments(edit)) {
    const overlaps =
      segment.outputEnd > range.start + RANGE_EPSILON &&
      segment.outputStart < range.end - RANGE_EPSILON;
    if (!overlaps) continue;
    const rate = segment.playbackRate ?? 1;
    if (playbackRate !== undefined && playbackRate !== rate) return undefined;
    playbackRate = rate;
  }
  return playbackRate;
}
