// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RotateCcw } from "lucide-react";

import { Button } from "../../../../components/base/button/button";
import { IconButton } from "../../../../components/base/button/icon-button";
import { NumberField } from "../../../../components/base/input-fields/number-field";
import { PillGroup } from "../../../../components/base/pill-group/pill-group";
import { Switch } from "../../../../components/base/switch/switch";
import { Text } from "../../../../components/base/text/text";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { SliderNumberField } from "../../../../components/shared/slider-number-field/slider-number-field";
import { t } from "../../../../i18n/i18n";
import { keyboardDefaultCenter } from "../../keyboard-effect/keyboard-effect-geometry";
import {
  EditorKind,
  KeyboardEffectAnimation,
  KeyboardEffectAppearance,
} from "../../types";
import { useToolPanelSnapshot } from "../use-tool-panel-snapshot";

const animationOptions = (): {
  id: KeyboardEffectAnimation;
  label: string;
}[] => [
  { id: "pop", label: t("editor-panels-keyboard-pop") },
  { id: "fade", label: t("editor-panels-keyboard-fade") },
  { id: "none", label: t("editor-panels-keyboard-none") },
];

const appearanceOptions = (): {
  id: KeyboardEffectAppearance;
  label: string;
}[] => [
  { id: "dark", label: t("editor-panels-keyboard-dark") },
  { id: "light", label: t("editor-panels-keyboard-light") },
];

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
    return <Text variant="footnote">{t("editor-panels-keyboard-empty")}</Text>;
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
      <ControlRow title={t("editor-panels-keyboard-show")}>
        {(controlProps) => (
          <Switch
            {...controlProps}
            isDisabled={isLocked}
            isSelected={keyboardEffects.bake}
            onChange={(bake) => {
              change({ keyboardEffects: { bake } });
            }}
          />
        )}
      </ControlRow>

      <ControlRow title={t("editor-panels-size")}>
        {(controlProps) => (
          <div {...controlProps} role="group">
            <SliderNumberField
              aria-label={t("editor-panels-size")}
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
        )}
      </ControlRow>

      <ControlRow title={t("editor-panels-position")}>
        {(controlProps) => (
          <div {...controlProps} className="flex gap-control" role="group">
            <IconButton
              aria-label={t("editor-panels-reset-position")}
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
              aria-label={t("editor-panels-shortcut-x")}
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
              aria-label={t("editor-panels-shortcut-y")}
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
        )}
      </ControlRow>

      <ControlRow title={t("editor-panels-keyboard-animation")}>
        {(controlProps) => (
          <div {...controlProps} role="group">
            <PillGroup
              aria-label={t("editor-panels-keyboard-animation")}
              display="label"
              isDisabled={isDisabled}
              items={animationOptions()}
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
        )}
      </ControlRow>

      <ControlRow title={t("editor-panels-keyboard-appearance")}>
        {(controlProps) => (
          <div {...controlProps} role="group">
            <PillGroup
              aria-label={t("editor-panels-keyboard-appearance")}
              display="label"
              isDisabled={isDisabled}
              items={appearanceOptions()}
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
        )}
      </ControlRow>

      <div className="flex justify-end gap-control">
        <Button
          isDisabled={isDisabled || !canRestoreShortcuts}
          onPress={() => {
            change({ restoreShortcuts: true });
          }}
        >
          {t("editor-panels-keyboard-restore")}
        </Button>
        <Button
          isDisabled={isDisabled}
          onPress={() => {
            change({ resetAllShortcuts: true });
          }}
        >
          {t("editor-panels-reset-all")}
        </Button>
      </div>
    </div>
  );
}
