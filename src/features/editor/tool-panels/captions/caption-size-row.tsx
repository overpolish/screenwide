// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { StepSlider } from "../../../../components/shared/step-slider/step-slider";

import type { TextSize } from "../../../../bindings/TextSize";

/** The steps from the template's size, smallest first; Medium is its own.
 * The twin of `TextSize` in `screenwide_text`, which says how large
 * each one draws. */
const STEPS: TextSize[] = ["small", "medium", "large", "extra-large"];

/** A size stepped through from the template's, as a text box's is. */
export function CaptionSizeRow({
  isDisabled,
  onChange,
  size,
  title,
}: {
  isDisabled: boolean;
  onChange: (size: TextSize) => void;
  size: TextSize;
  title: string;
}) {
  return (
    <ControlRow title={title}>
      {(controlProps) => (
        <div {...controlProps} role="group">
          <StepSlider
            aria-label={title}
            isDisabled={isDisabled}
            onChange={onChange}
            steps={STEPS}
            value={size}
          />
        </div>
      )}
    </ControlRow>
  );
}
