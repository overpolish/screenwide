// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Slider } from "../../base/slider/slider";

import { stepIndex } from "./step-index";

/**
 * A choice among a few steps, smallest first: an annotation's width or
 * blur, a caption's size. The steps need not be evenly spaced, or numbers
 * at all, so the knob runs over each one's place in the list and the ticks
 * say how many there are. A number saved before the steps lands the knob on
 * the nearest rather than nowhere.
 */
export function StepSlider<T extends number | string>({
  "aria-label": label,
  isDisabled,
  onChange,
  steps,
  value,
}: {
  "aria-label": string;
  onChange: (value: T) => void;
  steps: readonly T[];
  value: T;
  isDisabled?: boolean;
}) {
  return (
    <Slider
      aria-label={label}
      className="w-48"
      isDisabled={isDisabled}
      maxValue={steps.length - 1}
      minValue={0}
      onChange={(index) => {
        const step =
          steps[Math.min(steps.length - 1, Math.max(0, Math.round(index)))];
        onChange(step);
      }}
      showTicks
      step={1}
      value={stepIndex(steps, value)}
    />
  );
}
