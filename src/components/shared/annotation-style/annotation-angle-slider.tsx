// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SliderNumberField } from "../slider-number-field/slider-number-field";

/**
 * Where a counter's tail points, in degrees clockwise from east - the space
 * its grip drags in, and the space the document stores, only in radians.
 *
 * A degree at a time, rather than stepped to the eight aims Shift snaps a
 * drag to: the control has to be able to show whatever angle a free drag
 * left behind, and a knob that rounded to the nearest eighth would lie about
 * the mark it is dressing. The snaps stay where they are useful - on the
 * picture, where the hand is imprecise - and the field is where an exact
 * angle is typed.
 *
 * The track stops one degree short of the turn: 360 and 0 are the same aim,
 * and a control with both on it has a dead step at one end.
 */
export function AnnotationAngleSlider({
  isDisabled,
  onChange,
  value,
}: {
  /** The aim in radians, as the mark stores it. */
  onChange: (radians: number) => void;
  value: number;
  isDisabled?: boolean;
}) {
  const degrees = ((Math.round((value * 180) / Math.PI) % 360) + 360) % 360;
  return (
    <SliderNumberField
      aria-label="Angle"
      className="w-48"
      isDisabled={isDisabled}
      maxValue={359}
      minValue={0}
      onChange={(next) => {
        onChange((next * Math.PI) / 180);
      }}
      rightSection="°"
      step={1}
      value={degrees}
    />
  );
}
