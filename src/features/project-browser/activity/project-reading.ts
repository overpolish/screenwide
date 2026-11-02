// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  linesOf,
  type RecordingBeat,
  recordingBeats,
} from "../../editor/timeline/recording-strip/recording-beats";
import { recordingStripTime } from "../../editor/timeline/recording-strip/recording-strip-time";
import {
  recordedTime,
  type Span,
  type TimeMap,
} from "../../editor/timeline/timeline-scale";

import type { ProjectActivity } from "../types";

// A recording as the browser reads it: the beats the editor's strip finds
// in it, by the strip's own rules, so a card's overview and the strip
// agree; the lines said; the moments flagged; and how its recorded time
// maps to the time it plays in now, so what is drawn along the edit lines
// up with the scrub strip, which the editor composes along the edit. Times
// are in recorded seconds unless they say otherwise.

type ProjectLine = Span & { text: string };

type ProjectMomentMark = {
  at: number;
  /** The kind's colour, `#rrggbb`. */
  color: string;
  name: string;
  /** What its voice note says, once transcribed. */
  note: string | null;
};

export type ProjectReading = {
  beats: RecordingBeat[];
  lines: ProjectLine[];
  map: TimeMap;
  moments: ProjectMomentMark[];
};

export function projectReading(activity: ProjectActivity): ProjectReading {
  const seconds = (ms: number) => ms / 1000;
  const duration = seconds(activity.durationMs);
  const moments = activity.moments.map(({ color, name, note, sourceMs }) => ({
    at: seconds(sourceMs),
    color,
    name,
    note,
  }));
  const words = activity.words.map(({ endMs, startMs, text }) => ({
    end: seconds(endMs),
    start: seconds(startMs),
    text,
  }));
  const { beats } = recordingBeats({
    clicks: activity.activity.clicks.map(({ count, ms }) => ({
      at: seconds(ms),
      count,
    })),
    duration,
    moments: activity.moments.map(({ color, index, name, sourceMs }) => ({
      at: seconds(sourceMs),
      color,
      index,
      name,
    })),
    picture: activity.activity.picture.map(({ endMs, startMs, state }) => ({
      end: seconds(endMs),
      start: seconds(startMs),
      state,
    })),
    // The keyboard timeline is the editor's to work out; typing and the
    // rest are enough to see a recording's shape by.
    shortcuts: [],
    typing: activity.activity.typing.map(seconds),
    words,
  });
  return {
    beats,
    lines: linesOf(words).map(({ end, start, text }) => ({ end, start, text })),
    map: activity.edit
      ? recordingStripTime(activity.edit, duration).map
      : recordedTime(duration),
    moments,
  };
}

const within = (at: number) => (span: Span) =>
  at >= span.start && at < span.end;

/** The beat a recorded moment falls in, if any. */
export const beatAt = (reading: ProjectReading, at: number) =>
  reading.beats.find(({ span }) => within(at)(span));

/** The line being said at a recorded moment, if any. */
export const lineAt = (reading: ProjectReading, at: number) =>
  reading.lines.find(within(at));
