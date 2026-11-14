// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Span, type TimeMap } from "../timeline-scale";

/** The shortest scene a drag on the rail draws, in seconds as it plays. A
 * shorter drag grows to it, so a scene drawn too short still appears where
 * it was drawn rather than vanishing on release. */
export const LEAST_DRAWN_S = 1;

/**
 * The stretch a drag from `from` to `to` draws, either way round, held
 * inside `room`: where it is shorter than `least`, grown to it from where it
 * was pressed, the way it was dragged, so the scene starts under the press
 * (or ends there, drawn backwards); and slid back inside the room where
 * growing took it past an edge. Any unit, so long as all of them agree.
 */
export function drawnSceneSpan(
  { from, to }: { from: number; to: number },
  room: { from: number; to: number },
  least: number,
) {
  let start = Math.min(from, to);
  let end = Math.max(from, to);
  if (end - start < least) {
    if (to < from) start = end - least;
    else end = start + least;
    const slide = Math.max(room.from - start, 0) - Math.max(end - room.to, 0);
    start += slide;
    end += slide;
  }
  return { end: Math.min(end, room.to), start: Math.max(start, room.from) };
}

/**
 * The room a scene drawn from `moment` has as the timeline plays, in played
 * seconds: from the end of the scene or gap before it to the start of the
 * one after, within what plays. `taken` and `moment` are in the rail's
 * seconds, which `map` lays out as they play.
 */
export function playedRoom(
  taken: readonly Span[],
  map: TimeMap,
  moment: number,
) {
  const at = map.toPlayed(moment);
  const played = taken.map(({ end, start }) => ({
    end: map.toPlayed(end, true),
    start: map.toPlayed(start),
  }));
  return {
    from: Math.max(
      0,
      ...played.filter(({ end }) => end <= at).map(({ end }) => end),
    ),
    to: Math.min(
      map.duration,
      ...played.filter(({ start }) => start >= at).map(({ start }) => start),
    ),
  };
}

/** The scene a drag along the rail from `from` to `to` draws, in recorded
 * seconds: at least the shortest one as it plays, inside the room where it
 * was pressed among what is `taken`. */
export function draftedScene(
  { from, to }: { from: number; to: number },
  taken: readonly Span[],
  map: TimeMap,
): Span {
  const span = drawnSceneSpan(
    { from: map.toPlayed(from, from > to), to: map.toPlayed(to, to > from) },
    playedRoom(taken, map, from),
    LEAST_DRAWN_S,
  );
  return { end: map.toRecorded(span.end), start: map.toRecorded(span.start) };
}

/** Whether `moment`, in the rail's seconds, lies over one of `gaps`, where
 * nothing is drawn. */
export const isOverGap = (gaps: readonly Span[], moment: number) =>
  gaps.some(({ end, start }) => moment >= start && moment < end);
