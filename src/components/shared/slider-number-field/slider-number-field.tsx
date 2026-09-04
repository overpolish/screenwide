// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../lib/styling";
import { NumberField } from "../../base/input-fields/number-field";
import { Slider } from "../../base/slider/slider";

import type { NumberFieldProps } from "../../base/input-fields/number-field";

export type SliderNumberFieldProps = Pick<
  NumberFieldProps,
  "leftSection" | "rightSection" | "formatOptions" | "isDisabled"
> & {
  "aria-label": string;
  maxValue: number;
  minValue: number;
  onChange: (value: number) => void;
  value: number;
  className?: string;
  numberFieldClassName?: string;
  /** Soft bounds for the slider; typing and scrubbing use the full range. */
  sliderMaxValue?: number;
  sliderMinValue?: number;
  step?: number;
};

export function SliderNumberField({
  "aria-label": label,
  className,
  formatOptions,
  isDisabled,
  leftSection,
  maxValue,
  minValue,
  numberFieldClassName,
  onChange,
  rightSection,
  sliderMaxValue = maxValue,
  sliderMinValue = minValue,
  step = 1,
  value,
}: SliderNumberFieldProps) {
  if (
    import.meta.env.DEV &&
    (![minValue, maxValue, sliderMinValue, sliderMaxValue, step].every(
      Number.isFinite,
    ) ||
      minValue >= maxValue ||
      sliderMinValue < minValue ||
      sliderMaxValue > maxValue ||
      sliderMinValue >= sliderMaxValue ||
      step <= 0)
  ) {
    throw new Error(
      "SliderNumberField requires finite bounds with minValue <= sliderMinValue < sliderMaxValue <= maxValue and a positive finite step.",
    );
  }

  const changeValue = (nextValue: number) => {
    // An empty/invalid number draft must not erase the shared numeric value.
    if (isDisabled || !Number.isFinite(nextValue)) return;
    onChange(Math.min(maxValue, Math.max(minValue, nextValue)));
  };

  return (
    <div
      aria-label={label}
      className={cn("gap-section flex min-w-0 items-center", className)}
      onPointerDownCapture={(event) => {
        // Prevent native scrubbing from starting on a disabled Number Field.
        if (isDisabled) event.stopPropagation();
      }}
      role="group"
    >
      <Slider
        aria-label={`${label} slider`}
        className="mx-control-inset min-w-0 flex-1"
        formatOptions={formatOptions}
        isDisabled={isDisabled}
        maxValue={sliderMaxValue}
        minValue={sliderMinValue}
        onChange={changeValue}
        step={step}
        value={Math.min(sliderMaxValue, Math.max(sliderMinValue, value))}
      />
      <NumberField
        aria-label={`${label} value`}
        className={cn("w-24 shrink-0", numberFieldClassName)}
        formatOptions={formatOptions}
        isDisabled={isDisabled}
        leftSection={leftSection}
        maxValue={maxValue}
        minValue={minValue}
        onChange={changeValue}
        rightSection={rightSection}
        showSteppers={false}
        size="compact"
        step={step}
        value={value}
      />
    </div>
  );
}
