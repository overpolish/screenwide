// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

/** The largest step, in percent of the track, that jumps rather than eases. */
const JUMP_PERCENT = 5;

const EASE = { duration: 0.1, ease: "easeOut" } as const;
const JUMP = { duration: 0 } as const;

/**
 * How a progress indicator moves to `percent`: small steps jump, larger ones
 * ease. A long save reports a percent or so several times a second, and
 * easing each step kept the indicator animating nearly all the time: the
 * window was redrawn every frame, and during an export that drew on the GPU
 * the export runs on. A step that small is a few pixels, which reads as
 * steady movement on its own.
 */
export function useProgressTransition(percent: number) {
  const [shown, setShown] = useState(percent);
  const [eases, setEases] = useState(false);
  if (percent !== shown) {
    setShown(percent);
    setEases(Math.abs(percent - shown) > JUMP_PERCENT);
  }
  return eases ? EASE : JUMP;
}

/** `value` as a percent of the span from `minValue` to `maxValue`. */
export const progressPercent = (
  value: number | undefined,
  minValue = 0,
  maxValue = 100,
) =>
  maxValue > minValue
    ? (((value ?? minValue) - minValue) / (maxValue - minValue)) * 100
    : 0;
