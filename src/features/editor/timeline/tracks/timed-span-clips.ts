// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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
