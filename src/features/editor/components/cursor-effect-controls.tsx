// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Checkbox } from "../../../components/base/checkbox/checkbox";
import { Slider } from "../../../components/base/slider/slider";
import { CursorEffectSettings } from "../types";

export function CursorEffectControls({
  isSaving,
  onChange,
  settings,
}: {
  isSaving: boolean;
  settings: CursorEffectSettings;
  onChange?: (settings: CursorEffectSettings) => void;
}) {
  const sizePercent = Number.isFinite(settings.sizePercent)
    ? settings.sizePercent
    : 100;
  const update = (change: Partial<CursorEffectSettings>) => {
    onChange?.({ ...settings, ...change });
  };
  return (
    <div className="flex flex-col gap-section">
      <Checkbox
        description="Hides it where it leaves the recorded area instead of drawing it over the frame."
        isDisabled={isSaving}
        isSelected={settings.clipAtVideoEdge}
        onChange={(clipAtVideoEdge) => {
          update({ clipAtVideoEdge });
        }}
      >
        Keep cursor inside the recording
      </Checkbox>
      <Checkbox
        description="Adds natural smoothing and momentum."
        isDisabled={isSaving}
        isSelected={settings.smoothMovement}
        onChange={(smoothMovement) => {
          update({ smoothMovement });
        }}
      >
        Smooth movement
      </Checkbox>
      <div className="flex flex-col gap-control">
        <div className="flex items-center justify-between gap-section text-body">
          <span>Cursor size</span>
          <span className="text-content-fg-secondary tabular-nums">
            {sizePercent.toString()}%
          </span>
        </div>
        <Slider
          aria-label="Cursor size"
          isDisabled={isSaving}
          maxValue={500}
          minValue={50}
          onChange={(nextSizePercent) => {
            update({ sizePercent: nextSizePercent });
          }}
          step={5}
          value={sizePercent}
        />
      </div>
      <Checkbox
        isDisabled={isSaving}
        isSelected={settings.motionBlur}
        onChange={(motionBlur) => {
          update({ motionBlur });
        }}
      >
        Motion blur
      </Checkbox>
      <Checkbox
        isDisabled={isSaving}
        isSelected={settings.clickAnimation}
        onChange={(clickAnimation) => {
          update({ clickAnimation });
        }}
      >
        Click animation
      </Checkbox>
    </div>
  );
}
