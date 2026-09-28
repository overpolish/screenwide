// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SliderNumberField } from "../slider-number-field/slider-number-field";

/** How round a redaction's or a shape's corners are, as a percentage of its
 * box's shorter side from 0 to 50. */
export function AnnotationRadiusField({
  isDisabled,
  onChange,
  value,
}: {
  onChange: (radius: number) => void;
  value: number;
  isDisabled?: boolean;
}) {
  return (
    <SliderNumberField
      aria-label="Radius"
      className="w-48"
      formatOptions={{
        maximumFractionDigits: 1,
        minimumFractionDigits: 1,
      }}
      isDisabled={isDisabled}
      maxValue={50}
      minValue={0}
      onChange={onChange}
      rightSection="%"
      step={0.1}
      value={value}
    />
  );
}
