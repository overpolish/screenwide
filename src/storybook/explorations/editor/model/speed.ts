// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Look } from "../recording";

import { type Pause } from "./timeline-model";

// How fast give may play, and how speeds are said.

/** The slowest give plays, and the fastest the app's player plays
 * (`validate_playback_rate`, 0.25× to 4×). */
export const SLOWEST = 0.5;
const FASTEST = 4;

/** How fast give plays at most, by what the picture is: a still screen or
 * a game's frozen menu hides any speed; motion blurs past twice its pace;
 * a face, even still, looks hurried past a quarter faster. */
export const CAPS: Record<Look, { moving: number; still: number }> = {
  camera: { moving: 1.25, still: 1.25 },
  game: { moving: 2, still: FASTEST },
  screen: { moving: 2, still: FASTEST },
};

/** How a pause retimed to `seconds` plays: what is said in it as it was,
 * its give no faster than `look` allows and no slower than half speed. The
 * seconds its still and its moving give each play. */
export function ownPlay(pause: Pause, seconds: number, look: Look) {
  const cap = CAPS[look];
  const rigid = pause.end - pause.start - pause.still - pause.moving;
  const give = Math.min(
    (pause.still + pause.moving) / SLOWEST,
    Math.max(
      seconds - rigid,
      pause.still / cap.still + pause.moving / cap.moving,
    ),
  );
  // Shared evenly, unless that hurries what moves past its cap: then it
  // goes at its cap and the still give takes the rest.
  const even = give / Math.max(pause.still + pause.moving, 1e-9);
  const moving = Math.max(pause.moving * even, pause.moving / cap.moving);
  return { moving, still: give - moving, total: rigid + give };
}

export const ownLength = (pause: Pause, seconds: number, look: Look) =>
  ownPlay(pause, seconds, look).total;

/** How fast a stretch plays against real time, as a multiple (2.5 is two
 * and a half times as fast, 0.5 half speed), or null where it plays as
 * recorded. A length in seconds says nothing to someone judging by
 * watching; how much it was hurried or slowed does. */
export const speedOf = (recorded: number, plays: number) => {
  if (plays <= 0) return null;
  const times = recorded / plays;
  return Math.abs(times - 1) < 0.05 ? null : times;
};

/** A speed as people say it: "2.5×", "0.5×". */
export const speedLabel = (times: number) =>
  `${times < 10 ? times.toFixed(1) : times.toFixed(0)}×`;
