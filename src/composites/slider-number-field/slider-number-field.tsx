// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RotateCcw } from "lucide-react";

import { t } from "../../i18n/i18n";
import { cn } from "../../lib/styling";
import { IconButton } from "../../primitives/button/icon-button";
import { NumberField } from "../../primitives/input-fields/number-field";
import { Slider } from "../../primitives/slider/slider";

// The value reads as text beside the slider and is a field only under the
// pointer or while typed into, so a panel of sliders does not read as a
// form, yet an exact value is one click away.

type SliderNumberFieldProps = {
  "aria-label": string;
  maxValue: number;
  minValue: number;
  onChange: (value: number) => void;
  value: number;
  /** Width of the slider and value; a reset takes its own room before it. */
  className?: string;
  /** Offers a reset while the value differs from it. */
  defaultValue?: number;
  isDisabled?: boolean;
  /** Soft bounds for the slider; typing and scrubbing use the full range. */
  sliderMaxValue?: number;
  sliderMinValue?: number;
  step?: number;
  unit?: string;
};

export function SliderNumberField({
  "aria-label": label,
  className,
  defaultValue,
  isDisabled,
  maxValue,
  minValue,
  onChange,
  sliderMaxValue = maxValue,
  sliderMinValue = minValue,
  step = 1,
  unit,
  value,
}: SliderNumberFieldProps) {
  if (import.meta.env.DEV) {
    const ordered =
      minValue <= sliderMinValue &&
      sliderMinValue < sliderMaxValue &&
      sliderMaxValue <= maxValue;
    if (!ordered || !(step > 0))
      throw new Error(
        "SliderNumberField needs minValue <= sliderMinValue < sliderMaxValue <= maxValue and a positive step.",
      );
  }

  const changeValue = (next: number) => {
    // A field emptied while typing must not erase the value.
    if (isDisabled || !Number.isFinite(next)) return;
    onChange(Math.min(maxValue, Math.max(minValue, next)));
  };

  // As many decimals as the step has, always shown, so 0.1 reads 8.0.
  const decimals = String(step).split(".")[1]?.length ?? 0;
  const formatOptions = {
    maximumFractionDigits: decimals,
    minimumFractionDigits: decimals,
  };
  const isChanged = defaultValue !== undefined && value !== defaultValue;

  return (
    <div
      aria-label={label}
      className="flex min-w-0 items-center gap-inline"
      onPointerDownCapture={(event) => {
        // Native scrubbing would otherwise start on a disabled field.
        if (isDisabled) event.stopPropagation();
      }}
      role="group"
    >
      {/* Before the slider and outside its width, so neither the slider nor
          the value moves or shrinks when it appears. */}
      {isChanged ? (
        <IconButton
          aria-label={t("controls-reset", { label })}
          isDisabled={isDisabled}
          onPress={() => {
            onChange(defaultValue);
          }}
          variant="ghost"
        >
          <RotateCcw />
        </IconButton>
      ) : null}
      <div className={cn("flex min-w-0 items-center gap-inline", className)}>
        <Slider
          aria-label={t("controls-slider", { label })}
          className="min-w-0 flex-1"
          formatOptions={formatOptions}
          isDisabled={isDisabled}
          maxValue={sliderMaxValue}
          minValue={sliderMinValue}
          onChange={changeValue}
          step={step}
          value={Math.min(sliderMaxValue, Math.max(sliderMinValue, value))}
        />
        <NumberField
          aria-label={t("controls-slider-value", { label })}
          // Pulled out by its padding, so the value lines up with the column's
          // end and the frame shown on hover reaches past it instead.
          className="-me-inset-sm w-20 shrink-0"
          formatOptions={formatOptions}
          isDisabled={isDisabled}
          material="quiet"
          maxValue={maxValue}
          minValue={minValue}
          onChange={changeValue}
          rightSection={unit}
          step={step}
          value={value}
        />
      </div>
    </div>
  );
}
