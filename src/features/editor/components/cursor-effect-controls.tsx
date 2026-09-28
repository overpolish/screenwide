// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Switch } from "../../../components/base/switch/switch";
import { ControlRow } from "../../../components/shared/control-row/control-row";
import { SliderNumberField } from "../../../components/shared/slider-number-field/slider-number-field";
import { CursorEffectSettings } from "../types";

type CursorToggleKey = {
  [K in keyof CursorEffectSettings]: CursorEffectSettings[K] extends boolean
    ? K
    : never;
}[keyof CursorEffectSettings];

export function CursorEffectControls({
  isDisabled,
  onChange,
  settings,
}: {
  isDisabled: boolean;
  settings: CursorEffectSettings;
  onChange?: (settings: CursorEffectSettings) => void;
}) {
  const sizePercent = Number.isFinite(settings.sizePercent)
    ? settings.sizePercent
    : 100;
  const update = (change: Partial<CursorEffectSettings>) => {
    onChange?.({ ...settings, ...change });
  };
  const toggle = (key: CursorToggleKey, title: string) => (
    <ControlRow title={title}>
      {(controlProps) => (
        <Switch
          {...controlProps}
          isDisabled={isDisabled}
          isSelected={settings[key]}
          onChange={(selected) => {
            update({ [key]: selected });
          }}
        />
      )}
    </ControlRow>
  );
  return (
    <div className="flex flex-col gap-section">
      {toggle("clipAtVideoEdge", "Keep cursor inside the recording")}
      {toggle("smoothMovement", "Smooth movement")}
      <ControlRow title="Size">
        {(controlProps) => (
          <div {...controlProps} role="group">
            <SliderNumberField
              aria-label="Cursor size"
              className="w-48"
              isDisabled={isDisabled}
              maxValue={500}
              minValue={50}
              onChange={(nextSizePercent) => {
                update({ sizePercent: nextSizePercent });
              }}
              rightSection="%"
              step={5}
              value={sizePercent}
            />
          </div>
        )}
      </ControlRow>
      {toggle("motionBlur", "Motion blur")}
      {toggle("clickAnimation", "Click animation")}
    </div>
  );
}
