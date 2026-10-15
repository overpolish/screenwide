// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { recordingTimelineLengthMs } from "../editing/recording-timeline-time";

/**
 * A clip that holds a stretch of the edited video to itself, as scenes and
 * the camera's background ranges do: timed on the edited video, kept in
 * order, and never overlapping another clip of its lane.
 */
export type TimedSpanClip = { endMs: number; id: string; startMs: number };

/** The clip playing `timelineMs` into the edited video, or null between
 * clips. */
export const timedSpanClipAt = <Clip extends TimedSpanClip>(
  clips: readonly Clip[],
  timelineMs: number,
) =>
  clips.find((clip) => clip.startMs <= timelineMs && timelineMs < clip.endMs) ??
  null;

/** Where the clip `id` may reach without running into its neighbours, on a
 * timeline `lengthMs` long. */
const roomAround = <Clip extends TimedSpanClip>(
  clips: readonly Clip[],
  id: string,
  lengthMs: number,
) => {
  const index = clips.findIndex((clip) => clip.id === id);
  return {
    clip: index < 0 ? null : clips[index],
    from: clips[index - 1]?.endMs ?? 0,
    to: clips[index + 1]?.startMs ?? lengthMs,
  };
};

/**
 * A new clip at `atMs` on a timeline `lengthMs` long, `defaultMs` long where
 * the free stretch around the playhead allows: it runs forward first and,
 * near the end of the video or the next clip, reaches back for the rest.
 * `make` dresses the span as a clip. Null where the playhead is inside a clip
 * or the stretch is shorter than `minimumMs`.
 */
export const insertTimedSpanClip = <Clip extends TimedSpanClip>({
  atMs,
  clips,
  defaultMs,
  lengthMs,
  make,
  minimumMs,
}: {
  atMs: number;
  clips: readonly Clip[];
  defaultMs: number;
  lengthMs: number;
  make: (span: { endMs: number; startMs: number }) => Clip;
  minimumMs: number;
}) => {
  if (timedSpanClipAt(clips, atMs)) return null;
  const length = Math.round(lengthMs);
  const at = Math.max(0, Math.min(length, Math.round(atMs)));
  const from = Math.max(
    0,
    ...clips.flatMap((clip) => (clip.endMs <= at ? [clip.endMs] : [])),
  );
  const after = clips.find((clip) => clip.startMs >= at);
  const end = Math.min(at + defaultMs, after?.startMs ?? length);
  const start = Math.max(from, Math.min(at, end - defaultMs));
  if (end - start < minimumMs) return null;
  const clip = make({ endMs: end, startMs: start });
  return {
    clip,
    clips: [...clips, clip].sort((a, b) => a.startMs - b.startMs),
  };
};

/**
 * The clip `id` slid `deltaOutput`, a share of the timeline, along it. The
 * other clips stay where they are and the moved one passes over them: it
 * lands in the free stretch nearest to where its middle was carried, keeping
 * how long it plays for, and where that stretch is shorter it fills the
 * stretch instead. A stretch shorter than `minimumMs` is passed over.
 */
export const moveTimedSpanClip = <Clip extends TimedSpanClip>({
  clips,
  deltaOutput,
  edit,
  id,
  minimumMs,
  sourceDurationMs,
}: {
  clips: readonly Clip[];
  deltaOutput: number;
  edit: RecordingTimelineEdit;
  id: string;
  minimumMs: number;
  sourceDurationMs: number;
}): Clip[] => {
  const clip = clips.find((item) => item.id === id);
  const lengthMs = Math.round(
    recordingTimelineLengthMs(edit, sourceDurationMs),
  );
  if (!clip || lengthMs <= 0) return [...clips];
  const others = clips.filter((item) => item.id !== id);
  const length = clip.endMs - clip.startMs;
  const middle = Math.max(
    length / 2,
    Math.min(
      lengthMs - length / 2,
      (clip.startMs + clip.endMs) / 2 + deltaOutput * lengthMs,
    ),
  );
  // The free stretch nearest the carried middle; the first of two equally
  // near wins.
  let gap: { distance: number; from: number; to: number } | null = null;
  let from = 0;
  for (const [start, end] of [
    ...others.map((item) => [item.startMs, item.endMs]),
    [lengthMs, lengthMs],
  ]) {
    if (start - from >= minimumMs) {
      const distance = Math.max(0, from - middle, middle - start);
      if (!gap || distance < gap.distance) gap = { distance, from, to: start };
    }
    from = end;
  }
  if (!gap) return [...clips];
  let placed = { endMs: gap.to, startMs: gap.from };
  if (gap.to - gap.from > length) {
    const startMs = Math.round(
      Math.max(gap.from, Math.min(gap.to - length, middle - length / 2)),
    );
    placed = { endMs: startMs + length, startMs };
  }
  return [...others, { ...clip, ...placed }].sort(
    (a, b) => a.startMs - b.startMs,
  );
};

/**
 * The clip `id` with one edge taken to `output`, a share of the timeline,
 * held between its neighbour and `minimumMs`, or the clip's own length where
 * it is shorter already.
 */
export const resizeTimedSpanClip = <Clip extends TimedSpanClip>({
  clips,
  edge,
  edit,
  id,
  minimumMs,
  output,
  sourceDurationMs,
}: {
  clips: readonly Clip[];
  edge: "endMs" | "startMs";
  edit: RecordingTimelineEdit;
  id: string;
  minimumMs: number;
  output: number;
  sourceDurationMs: number;
}): Clip[] => {
  const lengthMs = Math.round(
    recordingTimelineLengthMs(edit, sourceDurationMs),
  );
  const { clip, from, to } = roomAround(clips, id, lengthMs);
  if (!clip) return [...clips];
  const at = Math.round(Math.max(0, Math.min(1, output)) * lengthMs);
  const minimum = Math.min(minimumMs, clip.endMs - clip.startMs);
  const next =
    edge === "startMs"
      ? { startMs: Math.max(from, Math.min(at, clip.endMs - minimum)) }
      : { endMs: Math.min(to, Math.max(at, clip.startMs + minimum)) };
  return clips.map((item) => (item.id === id ? { ...item, ...next } : item));
};
