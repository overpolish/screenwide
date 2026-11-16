// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** The room, in px, kept between one shortcut's keys and the next's: wider
 * than the room between keys, so each reads as its own. */
const GAP_PX = 10;

/** A shortcut's tick and how wide its keys are drawn, in px. */
export type ShortcutPlace = { keysPx: number; x: number };

/**
 * Where each shortcut's keys start on a strip `width` px wide, given in the
 * order the shortcuts were pressed. Pressed close together, a shortcut's
 * keys are set after the keys before them, a clear gap apart, so they sit
 * side by side rather than over each other or run together. Keys pressed in
 * the last moments would run past the strip's end, as if there were more
 * recording there, so they end at it instead, and those before them are
 * drawn back to stay clear. A strip too full to hold every shortcut's keys
 * keeps them all on it: the first are drawn over each other at its start
 * rather than past it.
 */
export function placeShortcutKeys(
  places: ShortcutPlace[],
  width: number,
): number[] {
  let reached = -Infinity;
  const starts = places.map(({ keysPx, x }) => {
    const from = Math.max(x, reached + GAP_PX);
    reached = from + keysPx;
    return from;
  });
  let limit = width;
  for (let index = places.length - 1; index >= 0; index -= 1) {
    starts[index] = Math.max(
      0,
      Math.min(starts[index], limit - places[index].keysPx),
    );
    limit = starts[index] - GAP_PX;
  }
  return starts;
}
