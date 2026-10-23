// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  signedAngle,
  wrapAngle,
} from "../../primitives/angle-dial/angle-geometry";

/** Between a dial's twelve o'clock and the document's east. */
const QUARTER_TURN = 90;

/**
 * A dial reads clockwise from twelve o'clock and the document stores radians
 * clockwise from east, a quarter turn apart. Converting rather than showing
 * the document's own zero is what lets a dial's aim stand where the
 * annotation's tail stands on the picture.
 */
export const dialDegrees = (radians: number) =>
  wrapAngle(Math.round((radians * 180) / Math.PI) + QUARTER_TURN);

/** The reverse, signed the way `atan2` leaves an angle a drag produced. */
export const radiansFromDial = (degrees: number) =>
  (signedAngle(degrees - QUARTER_TURN) * Math.PI) / 180;
