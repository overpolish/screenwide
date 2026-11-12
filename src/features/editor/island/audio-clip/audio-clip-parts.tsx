// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Volume1, Volume2 } from "lucide-react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { NumberField } from "../../../../primitives/input-fields/number-field";
import { OptionToggle } from "../../../annotations/options/option-toggle";
import { IslandFigure } from "../island-figure";

/** How loud the clip plays, in dB from the level its kind plays at by
 * default, with its reset while it is off that level. */
export function AudioClipVolumeRow({
  onChange,
  volumeDb,
}: {
  onChange: (volumeDb: number) => void;
  volumeDb: number;
}) {
  return (
    <ControlRow title={t("editor-island-audio-volume")}>
      {(controlProps) => (
        <IslandFigure
          controlProps={controlProps}
          isDefault={volumeDb === 0}
          onReset={() => {
            onChange(0);
          }}
          resetLabel={t("editor-island-audio-reset-volume")}
        >
          <NumberField
            aria-label={t("editor-island-audio-volume")}
            maxValue={12}
            minValue={-60}
            onChange={onChange}
            reservedLength={3}
            rightSection="dB"
            step={1}
            value={volumeDb}
          />
        </IslandFigure>
      )}
    </ControlRow>
  );
}

/** Whether the clip comes down while the microphone speaks: the sound
 * quietened while it does, or at its full level throughout. */
export function MakeWayToggle({
  makesWay,
  onChange,
}: {
  makesWay: boolean;
  onChange: (makesWay: boolean) => void;
}) {
  return (
    <OptionToggle
      glyph={(isOn) =>
        isOn ? (
          <Volume1 aria-hidden className="size-icon" />
        ) : (
          <Volume2 aria-hidden className="size-icon" />
        )
      }
      isOn={makesWay}
      label={t("editor-island-audio-make-way")}
      onChange={onChange}
    />
  );
}
