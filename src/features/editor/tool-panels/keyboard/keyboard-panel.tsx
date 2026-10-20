// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../../composites/control-row/control-row";
import { StepSlider } from "../../../../composites/step-slider/step-slider";
import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { PositionGrid } from "../../../../primitives/position-grid/position-grid";
import { Switch } from "../../../../primitives/switch/switch";
import { Text } from "../../../../primitives/text/text";
import {
  EditorKind,
  KeyboardEffectAnimation,
  KeyboardEffectAppearance,
} from "../../types";
import { OVERLAY_SIZES, placementOptions } from "../placement-options";
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
 * settings are: one size, a step, and one of the nine places captions offer,
 * for every shortcut. A press on the shortcuts in the preview opens this
 * panel. Restoring the deleted shortcuts ends the panel rather than sitting
 * among the settings.
 */
export function KeyboardPanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);
  const { canRestoreShortcuts, hasKeyboardData, isLocked, keyboardEffects } =
    snapshot;

  if (!hasKeyboardData) {
    return <Text variant="footnote">{t("editor-panels-keyboard-empty")}</Text>;
  }

  const isDisabled = isLocked || !keyboardEffects.bake;

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
            <StepSlider
              aria-label={t("editor-panels-size")}
              isDisabled={isDisabled}
              onChange={(size) => {
                change({ keyboardEffects: { size } });
              }}
              steps={OVERLAY_SIZES}
              value={keyboardEffects.size}
            />
          </div>
        )}
      </ControlRow>

      <ControlRow title={t("editor-panels-position")}>
        {(controlProps) => (
          <PositionGrid
            aria-labelledby={controlProps["aria-labelledby"]}
            isDisabled={isDisabled}
            items={placementOptions()}
            onChange={(position) => {
              change({ keyboardEffects: { position } });
            }}
            value={keyboardEffects.position}
          />
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
      </div>
    </div>
  );
}
