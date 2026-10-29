// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { NumberField } from "../../../../primitives/input-fields/number-field";
import { IslandFigure } from "../island-figure";

/** How loud a track plays, in decibels against the level it was recorded
 * at: a typed figure named by the track, with its reset while it is off
 * that level. */
export function SoundVolumeRow({
  decibels,
  label,
  onChange,
}: {
  decibels: number;
  /** The track's own name, such as Microphone. */
  label: string;
  onChange: (decibels: number) => void;
}) {
  return (
    <ControlRow title={label}>
      {(controlProps) => (
        <IslandFigure
          controlProps={controlProps}
          isDefault={decibels === 0}
          onReset={() => {
            onChange(0);
          }}
          resetLabel={t("editor-island-sound-reset-volume", { track: label })}
        >
          <NumberField
            aria-label={t("editor-island-sound-volume", { track: label })}
            maxValue={12}
            minValue={-60}
            onChange={onChange}
            rightSection="dB"
            step={1}
            value={decibels}
          />
        </IslandFigure>
      )}
    </ControlRow>
  );
}
