// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../components/base/button/button";
import { Checkbox } from "../../../components/base/checkbox/checkbox";
import { NumberField } from "../../../components/base/input-fields/number-field";
import { PillGroup } from "../../../components/base/pill-group/pill-group";
import { Slider } from "../../../components/base/slider/slider";
import {
  keyboardDefaultCenter,
  keyboardMaximumSizePercent,
} from "../keyboard-effect-geometry";
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
  canRestoreShortcuts = false,
  isSaving,
  maximumWidthUnits,
  onChange,
  onResetShortcuts,
  onRestoreShortcuts,
  outputDimensions,
  settings,
}: {
  isSaving: boolean;
  outputDimensions: { height: number; width: number };
  settings: KeyboardEffectSettings;
  canRestoreShortcuts?: boolean;
  maximumWidthUnits?: number | null;
  onChange?: (settings: KeyboardEffectSettings) => void;
  onResetShortcuts?: () => void;
  onRestoreShortcuts?: () => void;
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
  const position = keyboardDefaultCenter({
    positionXPercent: settings.positionXPercent,
    positionYPercent: settings.positionYPercent,
    sizePercent,
  });

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
          <span className="text-xs">Bake shortcuts into recording</span>
          <span className="text-xs text-muted">
            Show captured keyboard shortcuts
          </span>
        </span>
      </Checkbox>
      <div className="gap-control flex flex-col">
        <div className="gap-section flex items-center justify-between text-xs">
          <span>Shortcut size</span>
          <span className="text-muted tabular-nums">
            {sizePercent.toString()}%
          </span>
        </div>
        <Slider
          aria-label="Shortcut size"
          isDisabled={isSaving || !settings.bake}
          maxValue={maximumSizePercent}
          minValue={Math.min(50, maximumSizePercent)}
          onChange={(value) => {
            update({ sizePercent: value });
          }}
          step={5}
          value={sizePercent}
        />
      </div>
      <div className="flex items-center justify-between gap-3">
        <span className="text-xs text-content-fg">Position</span>
        <div className="flex gap-2">
          <NumberField
            aria-label="Shortcut X position"
            className="w-20"
            isDisabled={isSaving || !settings.bake}
            leftSection="X"
            maxValue={100}
            minValue={0}
            onChange={(positionXPercent) => {
              update({ positionXPercent });
            }}
            rightSection="%"
            showSteppers={false}
            size="compact"
            step={1}
            value={position.x * 100}
          />
          <NumberField
            aria-label="Shortcut Y position"
            className="w-20"
            isDisabled={isSaving || !settings.bake}
            leftSection="Y"
            maxValue={100}
            minValue={0}
            onChange={(positionYPercent) => {
              update({ positionYPercent });
            }}
            rightSection="%"
            showSteppers={false}
            size="compact"
            step={1}
            value={position.y * 100}
          />
        </div>
      </div>
      <div className="flex items-center justify-between gap-3">
        <span className="text-xs text-content-fg">Animation</span>
        <PillGroup
          aria-label="Shortcut animation"
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
          aria-label="Shortcut appearance"
          display="label"
          isDisabled={isSaving || !settings.bake}
          items={appearanceOptions}
          onSelectionChange={(appearance) => {
            update({ appearance: appearance as KeyboardEffectAppearance });
          }}
          selected={settings.appearance}
        />
      </div>
      <div className="flex justify-center gap-2">
        <Button
          isDisabled={isSaving || !canRestoreShortcuts || !onRestoreShortcuts}
          onPress={onRestoreShortcuts}
          size="compact"
        >
          Restore all shortcuts
        </Button>
        <Button
          isDisabled={isSaving || !onResetShortcuts}
          onPress={onResetShortcuts}
          size="compact"
        >
          Reset all
        </Button>
      </div>
    </div>
  );
}
