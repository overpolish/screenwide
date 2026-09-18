// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AngleDial } from "../../base/angle-dial/angle-dial";
import { signedAngle, wrapAngle } from "../../base/angle-dial/angle-geometry";
import { NumberField } from "../../base/input-fields/number-field";

/** Between the dial's twelve o'clock and the document's east. */
const QUARTER_TURN = 90;

/**
 * The dial reads clockwise from twelve o'clock and the document stores radians
 * clockwise from east, a quarter turn apart. Converting here rather than
 * showing the document's own zero is what lets the notch stand where the
 * annotation's tail stands on the picture and the field agree with the notch.
 */
const dialDegrees = (radians: number) =>
  wrapAngle(Math.round((radians * 180) / Math.PI) + QUARTER_TURN);

/** The reverse, signed the way `atan2` leaves an angle a drag produced. */
const radiansFromDial = (degrees: number) =>
  (signedAngle(degrees - QUARTER_TURN) * Math.PI) / 180;

/**
 * Where a counter's tail points: a dial with the aim under its notch, and the
 * degree beside it for an exact angle.
 *
 * A degree at a time, rather than stepped to the eight aims Shift snaps a drag
 * to: the control has to be able to show whatever angle a free drag left
 * behind, and a notch that rounded to the nearest eighth would lie about the
 * annotation it is dressing. The snaps stay where they are useful - on the
 * picture, and under a held Shift on the dial, where the hand is imprecise.
 *
 * The field commits as it is typed. The aim dresses the next annotation rather
 * than one on screen, so a value waiting behind an unfocused field is a value
 * the stroke it was typed for will not be drawn with - the press that draws is
 * on the picture, which is a window away and blurs nothing here.
 */
export function AnnotationAngleDial({
  isDisabled,
  onChange,
  value,
}: {
  /** The aim in radians, as the annotation stores it. `typed` marks the value
   * as entered rather than turned: it is one edit, and it has to reach whatever
   * reads the setting before the next stroke does, where a turn is a stream of
   * steps whose write can wait for the hand to settle. */
  onChange: (radians: number, typed: boolean) => void;
  value: number;
  isDisabled?: boolean;
}) {
  const degrees = dialDegrees(value);
  // A typed or scrubbed degree wraps like the dial's own: the conversion
  // folds it into one turn, so the field needs no bounds to stop at.
  const change = (next: number, typed: boolean) => {
    if (isDisabled || !Number.isFinite(next)) return;
    onChange(radiansFromDial(next), typed);
  };

  return (
    <div
      aria-label="Angle"
      className="flex shrink-0 items-center gap-control-inset"
      role="group"
    >
      <AngleDial
        aria-label="Angle"
        isDisabled={isDisabled}
        onChange={(next) => {
          change(next, false);
        }}
        value={degrees}
      />
      <NumberField
        aria-label="Angle value"
        className="w-16 shrink-0"
        isDisabled={isDisabled}
        onChange={(next) => {
          change(next, false);
        }}
        onTypedChange={(next) => {
          change(next, true);
        }}
        rightSection="°"
        showSteppers={false}
        step={1}
        value={degrees}
      />
    </div>
  );
}
