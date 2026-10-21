// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { Slider } from "../../../../primitives/slider/slider";

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
  const names: Record<TextSize, string> = {
    "extra-large": t("editor-panels-text-size-extra-large"),
    large: t("editor-panels-text-size-large"),
    medium: t("editor-panels-text-size-medium"),
    small: t("editor-panels-text-size-small"),
  };
  return (
    <ControlRow title={title}>
      {(controlProps) => (
        <div {...controlProps} role="group">
          <Slider
            aria-label={title}
            className="w-48"
            isDisabled={isDisabled}
            onChange={onChange}
            stepLabel={(step) => names[step]}
            steps={STEPS}
            value={size}
          />
        </div>
      )}
    </ControlRow>
  );
}
