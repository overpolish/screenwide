// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * A dial's space: degrees clockwise from twelve o'clock, the way a compass
 * and every rotation dial in an editor reads. A turn is 360 of them and the
 * value never leaves `[0, 360)`, so there is no end to run into.
 */
const TURN = 360;

/** `degrees` folded back into one turn. */
export const wrapAngle = (degrees: number) => ((degrees % TURN) + TURN) % TURN;

/** Half a turn: where an angle measured either side of zero folds over. */
const HALF_TURN = TURN / 2;

/**
 * `degrees` folded into the half turn either side of zero, the way `atan2`
 * leaves an angle - so a bearing can be stored as, or compared with, the
 * signed angle a drag produced.
 */
export const signedAngle = (degrees: number) =>
  wrapAngle(degrees + HALF_TURN) - HALF_TURN;

/**
 * `degrees` held to the nearest multiple of `step`, within one turn - so the
 * last step before the turn lands back on zero rather than on 360.
 */
export const snapAngle = (degrees: number, step: number) =>
  step > 0 ? wrapAngle(Math.round(degrees / step) * step) : wrapAngle(degrees);

/**
 * Where `point` lies as seen from the centre of `bounds`, in the dial's own
 * degrees. A press exactly on the centre has no direction in it, so it
 * reports none and leaves the dial where it was - the same rule the tail's
 * grip follows on the picture.
 */
export const angleAtPoint = (
  bounds: { height: number; left: number; top: number; width: number },
  point: { clientX: number; clientY: number },
) => {
  const x = point.clientX - (bounds.left + bounds.width / 2);
  const y = point.clientY - (bounds.top + bounds.height / 2);
  if (x === 0 && y === 0) return null;
  return wrapAngle(90 + (Math.atan2(y, x) * 180) / Math.PI);
};
