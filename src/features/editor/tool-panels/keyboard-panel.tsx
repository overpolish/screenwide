// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RotateCcw } from "lucide-react";

import { Button } from "../../../components/base/button/button";
import { IconButton } from "../../../components/base/button/icon-button";
import { NumberField } from "../../../components/base/input-fields/number-field";
import { PillGroup } from "../../../components/base/pill-group/pill-group";
import { Switch } from "../../../components/base/switch/switch";
import { Text } from "../../../components/base/text/text";
import { SliderNumberField } from "../../../components/shared/slider-number-field/slider-number-field";
import { keyboardDefaultCenter } from "../keyboard-effect-geometry";
import {
  EditorKind,
  KeyboardEffectAnimation,
  KeyboardEffectAppearance,
} from "../types";

import { useToolPanelSnapshot } from "./use-tool-panel-snapshot";

const animationOptions = [
  { id: "pop", label: "Pop" },
  { id: "fade", label: "Fade" },
  { id: "none", label: "None" },
] satisfies { id: KeyboardEffectAnimation; label: string }[];

const appearanceOptions = [
  { id: "dark", label: "Dark" },
  { id: "light", label: "Light" },
] satisfies { id: KeyboardEffectAppearance; label: string }[];

/**
 * The Keyboard tool's own controls: how every captured shortcut is drawn.
 *
 * Everything here is the whole recording's, the way the cursor panel's
 * settings are. One shortcut placed by hand is the Select tool's business and
 * is shown in the Selection panel instead, which is why the two ways back -
 * the deleted shortcuts, and the placements - end this panel rather than
 * sitting among the settings they undo.
 */
export function KeyboardPanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const {
    canRestoreShortcuts,
    hasKeyboardData,
    isLocked,
    keyboardEffects,
    keyboardMaximum,
  } = snapshot;

  if (!hasKeyboardData) {
    return (
      <Text variant="footnote">This recording has no keyboard shortcuts.</Text>
    );
  }

  const isDisabled = isLocked || !keyboardEffects.bake;
  // The slider needs a range to run over, so a canvas narrow enough that the
  // shortcut's own floor meets its ceiling still leaves one to scrub.
  const maximum = Math.max(keyboardMaximum, 10);
  const minimum = Math.min(50, maximum - 5);
  const sizePercent = Number.isFinite(keyboardEffects.sizePercent)
    ? Math.min(Math.max(keyboardEffects.sizePercent, minimum), maximum)
    : 100;
  // The shortcut sits where the recording drew it until it is moved, so the
  // fields show that resting place rather than an empty pair.
  const position = keyboardDefaultCenter({
    positionXPercent: keyboardEffects.positionXPercent,
    positionYPercent: keyboardEffects.positionYPercent,
    sizePercent,
  });

  return (
    <div className="flex flex-col gap-section">
      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">Show shortcuts</span>
        <Switch
          aria-label="Show shortcuts"
          isDisabled={isLocked}
          isSelected={keyboardEffects.bake}
          onChange={(bake) => {
            change({ keyboardEffects: { bake } });
          }}
        />
      </div>

      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">Size</span>
        <SliderNumberField
          aria-label="Size"
          className="w-48"
          isDisabled={isDisabled}
          maxValue={maximum}
          minValue={minimum}
          onChange={(next) => {
            change({ keyboardEffects: { sizePercent: next } });
          }}
          rightSection="%"
          step={5}
          value={sizePercent}
        />
      </div>

      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">Position</span>
        <div className="flex gap-control">
          <IconButton
            aria-label="Reset position"
            isDisabled={
              isDisabled ||
              (keyboardEffects.positionXPercent === undefined &&
                keyboardEffects.positionYPercent === undefined)
            }
            onPress={() => {
              change({ resetKeyboardPosition: true });
            }}
          >
            <RotateCcw />
          </IconButton>
          <NumberField
            aria-label="Shortcut X position"
            className="w-20"
            isDisabled={isDisabled}
            leftSection="X"
            maxValue={100}
            minValue={0}
            onChange={(positionXPercent) => {
              change({ keyboardEffects: { positionXPercent } });
            }}
            rightSection="%"
            showSteppers={false}
            step={1}
            value={position.x * 100}
          />
          <NumberField
            aria-label="Shortcut Y position"
            className="w-20"
            isDisabled={isDisabled}
            leftSection="Y"
            maxValue={100}
            minValue={0}
            onChange={(positionYPercent) => {
              change({ keyboardEffects: { positionYPercent } });
            }}
            rightSection="%"
            showSteppers={false}
            step={1}
            value={position.y * 100}
          />
        </div>
      </div>

      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">Animation</span>
        <PillGroup
          aria-label="Animation"
          display="label"
          isDisabled={isDisabled}
          items={animationOptions}
          onSelectionChange={(animation) => {
            change({
              keyboardEffects: {
                animation: animation as KeyboardEffectAnimation,
              },
            });
          }}
          selected={keyboardEffects.animation}
        />
      </div>

      <div className="flex items-center justify-between gap-section">
        <span className="text-body text-content-fg">Appearance</span>
        <PillGroup
          aria-label="Appearance"
          display="label"
          isDisabled={isDisabled}
          items={appearanceOptions}
          onSelectionChange={(appearance) => {
            change({
              keyboardEffects: {
                appearance: appearance as KeyboardEffectAppearance,
              },
            });
          }}
          selected={keyboardEffects.appearance}
        />
      </div>

      <div className="flex justify-end gap-control">
        <Button
          isDisabled={isDisabled || !canRestoreShortcuts}
          onPress={() => {
            change({ restoreShortcuts: true });
          }}
        >
          Restore all shortcuts
        </Button>
        <Button
          isDisabled={isDisabled}
          onPress={() => {
            change({ resetAllShortcuts: true });
          }}
        >
          Reset all
        </Button>
      </div>
    </div>
  );
}
