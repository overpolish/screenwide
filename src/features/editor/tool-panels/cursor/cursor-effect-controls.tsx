// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { Slider } from "../../../../primitives/slider/slider";
import { Switch } from "../../../../primitives/switch/switch";
import { CursorEffectSettings } from "../../types";
import { OVERLAY_SIZES, useOverlaySizeLabel } from "../placement-options";

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
  const overlaySizeLabel = useOverlaySizeLabel();
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
      <ControlRow title={t("editor-panels-size")}>
        {(controlProps) => (
          <div {...controlProps} role="group">
            <Slider
              aria-label={t("editor-panels-cursor-size")}
              className="w-48"
              isDisabled={isDisabled}
              onChange={(size) => {
                update({ size });
              }}
              stepLabel={overlaySizeLabel}
              steps={OVERLAY_SIZES}
              value={settings.size}
            />
          </div>
        )}
      </ControlRow>
      {toggle("clipAtVideoEdge", t("editor-panels-cursor-clip"))}
      {toggle("smoothMovement", t("editor-panels-cursor-smooth"))}
      {toggle("motionBlur", t("editor-panels-cursor-motion-blur"))}
      {toggle("clickAnimation", t("editor-panels-cursor-click-animation"))}
    </div>
  );
}
