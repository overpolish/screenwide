// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type TimelineCut } from "../cut-seams/cut-seams";
import {
  hasKeyboardSegmentEdits,
  recordingTimelineCuts,
} from "../editing/recording-timeline-cuts";
import {
  layoutRecordingTimelineSegments,
  type RecordingTimelineEdit,
  recordingTimelineRetainedDuration,
} from "../editing/recording-timeline-edit";
import { SLIVER_S } from "../strip/strip-split";
import { type Span, type TimeMap } from "../timeline-scale";

import type { RecordingTimelineSegment } from "../recording-timeline-types";

// How the strip reads the recording's timeline edit: its segments, in the
// order they play, as a map from recorded to played seconds; what was left
// out of them as cuts, each drawn where restoring it would put it back;
// where play jumps, which a move made; and where the recording is split
// with nothing taken out, which a press of the scissors made. Recorded
// moments a cut left out play, for no time, where the cut is.

/** Close enough, as a share of the recording, to be one moment. */
const SAME = 1e-9;
/** Close enough, in seconds, to be one moment. */
const TOLERANCE_S = 1e-6;

export type RecordingStripTime = {
  cuts: TimelineCut[];
  /** Where the parts that run across a jump are split, in recorded
   * seconds: both sides of every jump. */
  edges: number[];
  jumps: number[];
  map: TimeMap;
  /** What plays on without a jump, as stretches of played seconds: each
   * plays the recording forward, passing over only what is cut. */
  runs: Span[];
  splits: number[];
};

type Step = { end: number; from: number; rate: number; start: number };

/** A cut's id, from where it starts in the recording, so it keeps it while
 * the edit changes elsewhere. */
export const cutId = (sourceStart: number) =>
  `cut-${String(Math.round(sourceStart * 1e6))}`;

/** Whether any of `span`, in recorded seconds, plays: what a cut took out
 * whole plays for no time, at the cut's seam. Read within one stretch that
 * plays in order, as the strip's parts are once split at the jumps. */
export const isPlayed = (map: TimeMap, { end, start }: Span) =>
  map.toPlayed(end, true) - map.toPlayed(start) > TOLERANCE_S;

const rateOf = (segment: RecordingTimelineSegment) => segment.playbackRate ?? 1;

/** Whether two segments meeting in play could be one again: they meet in
 * the recording too, play at one speed, and no shortcut edit holds either
 * to its own. */
export const isJoinable = (
  edit: RecordingTimelineEdit,
  first: RecordingTimelineSegment,
  second: RecordingTimelineSegment,
) =>
  Math.abs(first.sourceEnd - second.sourceStart) < SAME &&
  rateOf(first) === rateOf(second) &&
  !hasKeyboardSegmentEdits(edit, first.id) &&
  !hasKeyboardSegmentEdits(edit, second.id);

/** Each segment's place in the recording: 0 for the one recorded first. */
export function sourceRanks(segments: readonly RecordingTimelineSegment[]) {
  const ranks = new Map<number, number>();
  [...segments]
    .sort((one, other) => one.sourceStart - other.sourceStart)
    .forEach((segment, index) => ranks.set(segment.id, index));
  return ranks;
}

/** The indexes of the segments play jumps to: those that do not follow,
 * in the recording, the one played before them. */
export function jumpIndexes(segments: readonly RecordingTimelineSegment[]) {
  const ranks = sourceRanks(segments);
  return segments.flatMap((segment, index) =>
    index > 0 &&
    ranks.get(segment.id) !== (ranks.get(segments[index - 1].id) ?? -2) + 1
      ? [index]
      : [],
  );
}

function timeMap(steps: Step[], duration: number): TimeMap {
  const played = steps.filter(({ rate }) => rate > 0);
  const cut = steps.filter(({ rate }) => rate === 0);
  // Segment edges are shares of the recording, so a moment named in
  // seconds can land a hair either side of the edge it names.
  const holds = (step: Step, recorded: number, isEnd: boolean) =>
    isEnd
      ? recorded > step.start + TOLERANCE_S &&
        recorded <= step.end + TOLERANCE_S
      : recorded >= step.start - TOLERANCE_S &&
        recorded < step.end - TOLERANCE_S;
  return {
    duration,
    toPlayed: (recorded, isEnd = false) => {
      const step =
        played.find((each) => holds(each, recorded, isEnd)) ??
        cut.find((each) => holds(each, recorded, isEnd)) ??
        cut.find((each) => recorded >= each.start && recorded <= each.end);
      if (!step) return recorded <= 0 ? 0 : duration;
      const within = Math.min(Math.max(recorded, step.start), step.end);
      return step.from + (within - step.start) * step.rate;
    },
    toRecorded: (at) => {
      // The last step to start by `at`: each step's end is summed again
      // from its length, so it can land a hair either side of where the
      // next starts, and a moment at a run's edge must not fall between
      // them or into the one before.
      const started = played.filter(({ from }) => from <= at + TOLERANCE_S);
      if (started.length === 0) return played.length > 0 ? played[0].start : 0;
      const step = started[started.length - 1];
      const within = Math.min(
        Math.max(at - step.from, 0),
        (step.end - step.start) * step.rate,
      );
      return step.start + within / step.rate;
    },
  };
}

/** The strip's reading of `edit`, for a recording `recorded` seconds long. */
export function recordingStripTime(
  edit: RecordingTimelineEdit,
  recorded: number,
): RecordingStripTime {
  const played = recordingTimelineRetainedDuration(edit) * recorded;
  const seconds = (share: number) => share * recorded;
  const layout = layoutRecordingTimelineSegments(edit);
  const cuts = recordingTimelineCuts(edit);
  const steps: Step[] = [
    ...layout.map((segment) => ({
      end: seconds(segment.sourceEnd),
      from: segment.outputStart * played,
      rate: 1 / rateOf(segment),
      start: seconds(segment.sourceStart),
    })),
    ...cuts.map((cut) => ({
      end: seconds(cut.sourceEnd),
      from: cut.outputPosition * played,
      rate: 0,
      start: seconds(cut.sourceStart),
    })),
  ];
  const { segments } = edit;
  const jumps = jumpIndexes(segments);
  const breaks = jumps.map((index) => layout[index].outputStart * played);
  return {
    cuts: cuts.map((cut) => ({
      end: seconds(cut.sourceEnd),
      id: cutId(cut.sourceStart),
      start: seconds(cut.sourceStart),
    })),
    edges: jumps.flatMap((index) => [
      seconds(segments[index - 1].sourceEnd),
      seconds(segments[index].sourceStart),
    ]),
    jumps: jumps.map((index) => seconds(segments[index].sourceStart)),
    map: timeMap(steps, played),
    runs: [0, ...breaks].map((start, index) => ({
      end: breaks[index] ?? played,
      start,
    })),
    splits: segments.flatMap((segment, index) =>
      index > 0 && isJoinable(edit, segments[index - 1], segment)
        ? [seconds(segment.sourceStart)]
        : [],
    ),
  };
}

/** A stretch's span in recorded seconds, from its span in played seconds,
 * inside one run. */
function recordedSpan(map: TimeMap, from: number, to: number) {
  const start = map.toRecorded(from);
  // Its end is the last moment it shows, which is in what plays before it.
  const end = map.toRecorded(Math.max(to - 1e-4, from)) + 1e-4;
  return { end: Math.max(end, start), start };
}

/** A clip's parts in recorded seconds, from its span in edited ms: one for
 * each run it plays across, so a clip played across a jump is drawn each
 * side of it. One span from its start to its end would take in what plays
 * elsewhere between them (a stretch moved away from inside it, drawn where
 * it plays now), or run backwards (one moved into it). */
export function recordedParts(
  map: TimeMap,
  runs: Span[],
  { endMs, startMs }: { endMs: number; startMs: number },
) {
  return runs.flatMap((run) => {
    const from = Math.max(startMs / 1000, run.start);
    const to = Math.min(endMs / 1000, run.end);
    return to - from >= SLIVER_S ? [recordedSpan(map, from, to)] : [];
  });
}

/** Where a recorded moment plays, in edited ms. */
export const playedMs = (map: TimeMap, seconds: number, isEnd?: boolean) =>
  Math.round(map.toPlayed(seconds, isEnd) * 1000);
