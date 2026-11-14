// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Snap } from "../bar-stack/use-stack-bar-drag";
import { baseOf } from "../strip/strip-part";
import { type Span } from "../timeline-scale";

/** The room a scene at a moment can have: from the end of what is taken
 * before it to the start of what is taken after, within the recording. */
function sceneRoom(taken: Span[], duration: number, moment: number) {
  return {
    from: Math.max(
      0,
      ...taken.filter(({ end }) => end <= moment).map(({ end }) => end),
    ),
    to: Math.min(
      duration,
      ...taken.filter(({ start }) => start >= moment).map(({ start }) => start),
    ),
  };
}

/** Where the scene `moving` goes, moved whole by `by` seconds: kept between
 * what is `taken` either side of it (the other scenes, other than its own
 * parts, and the gaps), and within the recording; with `snap`, its start or
 * else its end settles on what is in reach, which `guide` says. */
export function movedScene(
  {
    by,
    duration,
    moving,
    taken,
  }: {
    by: number;
    duration: number;
    moving: Span & { id: string };
    taken: (Span & { id?: string })[];
  },
  snap?: Snap,
): { guide: number | null; span: Span } {
  const others = taken.filter(
    ({ id }) => id === undefined || baseOf(id) !== baseOf(moving.id),
  );
  const before = sceneRoom(others, duration, moving.start).from;
  const after = sceneRoom(others, duration, moving.end).to;
  const length = moving.end - moving.start;
  const fit = (start: number) =>
    Math.min(Math.max(start, before), Math.max(before, after - length));
  const moved = fit(moving.start + by);
  const head = snap?.(moved);
  const tail = snap?.(moved + length);
  const start = head?.isSnapped
    ? head.seconds
    : tail?.isSnapped
      ? tail.seconds - length
      : moved;
  const settled = fit(start);
  return {
    guide:
      settled !== start
        ? null
        : head?.isSnapped
          ? head.seconds
          : tail?.isSnapped
            ? tail.seconds
            : null,
    span: { end: settled + length, start: settled },
  };
}
