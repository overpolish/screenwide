// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import {
  layoutRecordingTimelineSegments,
  RecordingTimelineEdit,
} from "../editing/recording-timeline-edit";

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
  /** Peak levels from 0 to 1. */
  waveform: number[];
};

/** A moment the edit keeps, and where it lands on the cut timeline. */
export type VisibleRecordingMoment = {
  moment: RecordingMoment;
  /** 0 to 1 along the cut timeline. */
  output: number;
  /** 0 to 1 along the recording. */
  source: number;
};

export const getRecordingMoments = (artifactId: number) =>
  invoke<RecordingMoment[]>("get_recording_moments", { artifactId });

/** The voice note of the `moment`th moment, as WAV bytes. */
export const getRecordingMomentNote = (artifactId: number, moment: number) =>
  invoke<ArrayBuffer>("get_recording_moment_note", { artifactId, moment });

/**
 * The moments whose part of the recording the edit keeps, in timeline order.
 * One in a part that was cut away is left out rather than moved to the join,
 * where it would point at something that is no longer there.
 */
export const visibleRecordingMoments = (
  edit: RecordingTimelineEdit,
  moments: readonly RecordingMoment[],
  sourceDurationMs: number,
): VisibleRecordingMoment[] => {
  if (sourceDurationMs <= 0) return [];
  const segments = layoutRecordingTimelineSegments(edit);
  const visible: VisibleRecordingMoment[] = [];
  for (const moment of moments) {
    const source = Math.min(1, Math.max(0, moment.sourceMs / sourceDurationMs));
    const segment = segments.find(
      (candidate) =>
        source >= candidate.sourceStart && source <= candidate.sourceEnd,
    );
    if (!segment) continue;
    const along =
      segment.sourceEnd > segment.sourceStart
        ? (source - segment.sourceStart) /
          (segment.sourceEnd - segment.sourceStart)
        : 0;
    visible.push({
      moment,
      output:
        segment.outputStart + along * (segment.outputEnd - segment.outputStart),
      source,
    });
  }
  return visible.sort((first, second) => first.output - second.output);
};

/** Close enough to the playhead to count as standing on a moment, so a
 * second press moves on rather than finding the moment it is already on. */
const SAME_POSITION_EPSILON = 1e-6;

/** Where the nearest moment after (`1`) or before (`-1`) `output` is, or
 * null when there is none that way. */
export const adjacentRecordingMoment = (
  visible: readonly VisibleRecordingMoment[],
  output: number,
  direction: 1 | -1,
): number | null => {
  const outputs = visible.map((entry) => entry.output);
  if (direction === 1)
    return outputs.find((at) => at > output + SAME_POSITION_EPSILON) ?? null;
  const before = outputs.filter((at) => at < output - SAME_POSITION_EPSILON);
  return before.length > 0 ? before[before.length - 1] : null;
};
