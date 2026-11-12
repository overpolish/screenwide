// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Snap } from "../bar-stack/use-stack-bar-drag";
import { baseOf } from "../strip/strip-part";
import { type Span } from "../timeline-scale";

/** How far, in px, a press on a chosen scene moves before it is a drag, so
 * a click that only chooses it never nudges it. */
export const SCENE_DRAG_PX = 3;

/** The room a scene at a moment can have: from the end of the one before
 * to the start of the one after, within the recording. */
export function sceneRoom(scenes: Span[], duration: number, moment: number) {
  return {
    from: Math.max(
      0,
      ...scenes.filter(({ end }) => end <= moment).map(({ end }) => end),
    ),
    to: Math.min(
      duration,
      ...scenes
        .filter(({ start }) => start >= moment)
        .map(({ start }) => start),
    ),
  };
}

/** Where the scene `moving` goes, moved whole by `by` seconds: kept between
 * the scenes either side of it, other than its own parts, and within the
 * recording; with `snap`, its start or else its end settles on what is in
 * reach, which `guide` says. */
export function movedScene(
  {
    by,
    duration,
    moving,
    scenes,
  }: {
    by: number;
    duration: number;
    moving: Span & { id: string };
    scenes: (Span & { id: string })[];
  },
  snap?: Snap,
): { guide: number | null; span: Span } {
  const others = scenes.filter(({ id }) => baseOf(id) !== baseOf(moving.id));
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
