// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Checkbox } from "../../../components/base/checkbox/checkbox";
import { PillGroup } from "../../../components/base/pill-group/pill-group";
import { Slider } from "../../../components/base/slider/slider";
import { keyboardMaximumSizePercent } from "../keyboard-effect-geometry";
import {
  KeyboardEffectAnimation,
  KeyboardEffectAppearance,
  KeyboardEffectSettings,
} from "../types";

import { useRecordingOutputDimensions } from "./recording-output-dimensions-channel";

const animationOptions = [
  { id: "pop", label: "Pop" },
  { id: "fade", label: "Fade" },
  { id: "none", label: "None" },
] satisfies { id: KeyboardEffectAnimation; label: string }[];

const appearanceOptions = [
  { id: "dark", label: "Dark" },
  { id: "light", label: "Light" },
] satisfies { id: KeyboardEffectAppearance; label: string }[];

export function KeyboardEffectControls({
  isSaving,
  maximumWidthUnits,
  onChange,
  outputDimensions,
  settings,
}: {
  isSaving: boolean;
  outputDimensions: { height: number; width: number };
  settings: KeyboardEffectSettings;
  maximumWidthUnits?: number | null;
  onChange?: (settings: KeyboardEffectSettings) => void;
}) {
  const update = (change: Partial<KeyboardEffectSettings>) => {
    onChange?.({ ...settings, ...change });
  };
  const liveDimensions = useRecordingOutputDimensions();
  const maximumSizePercent = keyboardMaximumSizePercent({
    ...(liveDimensions ?? outputDimensions),
    maximumWidthUnits,
  });
  const sizePercent = Number.isFinite(settings.sizePercent)
    ? Math.min(settings.sizePercent, maximumSizePercent)
    : 100;

  return (
    <div className="flex flex-col gap-4">
      <Checkbox
        isDisabled={isSaving}
        isSelected={settings.bake}
        onChange={(bake) => {
          update({ bake });
        }}
        size="sm"
      >
        <span className="flex flex-col">
          <span className="text-xs">Bake shortcuts into recording</span>
          <span className="text-xxs text-muted">
            Show captured keyboard shortcuts
          </span>
        </span>
      </Checkbox>
      <Slider
        isDisabled={isSaving || !settings.bake}
        label="Shortcut size"
        maxValue={maximumSizePercent}
        minValue={Math.min(50, maximumSizePercent)}
        onChange={(value) => {
          update({ sizePercent: value });
        }}
        renderValue={(value) => `${value.toString()}%`}
        step={5}
        value={sizePercent}
      />
      <div className="flex items-center justify-between gap-3">
        <span className="text-xs text-content-fg">Animation</span>
        <PillGroup
          ariaLabel="Shortcut animation"
          display="label"
          isDisabled={isSaving || !settings.bake}
          items={animationOptions}
          onSelectionChange={(animation) => {
            update({ animation: animation as KeyboardEffectAnimation });
          }}
          selected={settings.animation}
        />
      </div>
      <div className="flex items-center justify-between gap-3">
        <span className="text-xs text-content-fg">Appearance</span>
        <PillGroup
          ariaLabel="Shortcut appearance"
          display="label"
          isDisabled={isSaving || !settings.bake}
          items={appearanceOptions}
          onSelectionChange={(appearance) => {
            update({ appearance: appearance as KeyboardEffectAppearance });
          }}
          selected={settings.appearance}
        />
      </div>
    </div>
  );
}
