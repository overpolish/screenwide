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
    <div className="flex flex-col gap-4">
      <Checkbox
        isDisabled={isSaving}
        isSelected={settings.bake}
        onChange={(bake) => {
          update({ bake });
        }}
      >
        <span className="flex flex-col">
          <span className="text-xs">Bake cursor into recording</span>
          <span className="text-xs text-muted">Dynamic Screenwide cursor</span>
        </span>
      </Checkbox>
      <Checkbox
        isDisabled={isSaving || !settings.bake}
        isSelected={settings.clipAtVideoEdge}
        onChange={(clipAtVideoEdge) => {
          update({ clipAtVideoEdge });
        }}
      >
        <span className="text-xs">Clip at video edge</span>
      </Checkbox>
      <Checkbox
        isDisabled={isSaving || !settings.bake}
        isSelected={settings.smoothMovement}
        onChange={(smoothMovement) => {
          update({ smoothMovement });
        }}
      >
        <span className="flex flex-col">
          <span className="text-xs">Smooth movement</span>
          <span className="text-xs text-muted">
            Adds natural smoothing and momentum.
          </span>
        </span>
      </Checkbox>
      <div className="gap-control flex flex-col">
        <div className="gap-section flex items-center justify-between text-xs">
          <span>Cursor size</span>
          <span className="text-muted tabular-nums">
            {sizePercent.toString()}%
          </span>
        </div>
        <Slider
          aria-label="Cursor size"
          isDisabled={isSaving || !settings.bake}
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
        isDisabled={isSaving || !settings.bake}
        isSelected={settings.motionBlur}
        onChange={(motionBlur) => {
          update({ motionBlur });
        }}
      >
        <span className="text-xs">Motion blur</span>
      </Checkbox>
      <Checkbox
        isDisabled={isSaving || !settings.bake}
        isSelected={settings.clickAnimation}
        onChange={(clickAnimation) => {
          update({ clickAnimation });
        }}
      >
        <span className="text-xs">Click animation</span>
      </Checkbox>
    </div>
  );
}
