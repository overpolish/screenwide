// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** The shortest scene a drag on the rail draws, in seconds as it plays. A
 * shorter drag grows to it, so a scene drawn too short still appears where
 * it was drawn rather than vanishing on release. */
export const LEAST_DRAWN_S = 1;

/**
 * The stretch a drag from `from` to `to` draws, either way round, held
 * inside `room`: grown about its middle to `least` where it is shorter, and
 * slid back inside the room where growing took it past an edge. Any unit, so
 * long as all of them agree.
 */
export function drawnSceneSpan(
  { from, to }: { from: number; to: number },
  room: { from: number; to: number },
  least: number,
) {
  let start = Math.min(from, to);
  let end = Math.max(from, to);
  if (end - start < least) {
    const middle = (start + end) / 2;
    start = middle - least / 2;
    end = middle + least / 2;
    const slide = Math.max(room.from - start, 0) - Math.max(end - room.to, 0);
    start += slide;
    end += slide;
  }
  return { end: Math.min(end, room.to), start: Math.max(start, room.from) };
}
