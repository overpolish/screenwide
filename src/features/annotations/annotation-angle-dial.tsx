// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../i18n/i18n";
import { AngleDial } from "../../primitives/angle-dial/angle-dial";
import { NumberField } from "../../primitives/input-fields/number-field";

import { dialDegrees, radiansFromDial } from "./annotation-angle";

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
      aria-label={t("annotation-angle")}
      className="flex shrink-0 items-center gap-control-inset"
      role="group"
    >
      <AngleDial
        aria-label={t("annotation-angle")}
        isDisabled={isDisabled}
        onChange={(next) => {
          change(next, false);
        }}
        value={degrees}
      />
      <NumberField
        aria-label={t("annotation-angle-value")}
        className="w-16 shrink-0"
        isDisabled={isDisabled}
        onChange={(next) => {
          change(next, false);
        }}
        onTypedChange={(next) => {
          change(next, true);
        }}
        rightSection="°"
        step={1}
        value={degrees}
      />
    </div>
  );
}
