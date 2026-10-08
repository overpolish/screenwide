// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { GroupBox } from "../../../components/base/group-box/group-box";
import { Switch } from "../../../components/base/switch/switch";
import { ControlRow } from "../../../components/shared/control-row/control-row";
import { HotkeyField } from "../../../components/shared/hotkey-field/hotkey-field";
import { t } from "../../../i18n/i18n";

import type { RulerAction } from "../../../bindings/RulerAction";
import type { RulerSettings } from "../../../bindings/RulerSettings";
import type { ShortcutSettings } from "../../../bindings/ShortcutSettings";

const actions = (): {
  action: RulerAction;
  label: string;
  description?: string;
}[] => [
  { action: "toggleCrosshair", label: t("settings-ruler-crosshair") },
  {
    action: "copyColour",
    description: t("settings-ruler-copy-colour-description"),
    label: t("settings-ruler-copy-colour"),
  },
  {
    action: "deleteMeasurement",
    description: t("settings-ruler-delete-description"),
    label: t("settings-ruler-delete"),
  },
  {
    action: "copyMeasurement",
    description: t("settings-ruler-copy-description"),
    label: t("settings-ruler-copy"),
  },
  { action: "undo", label: t("settings-ruler-undo") },
  { action: "redo", label: t("settings-ruler-redo") },
  {
    action: "stampHorizontal",
    description: t("settings-ruler-stamp-description"),
    label: t("settings-ruler-stamp-horizontal"),
  },
  {
    action: "stampVertical",
    description: t("settings-ruler-stamp-description"),
    label: t("settings-ruler-stamp-vertical"),
  },
  {
    action: "guideVertical",
    description: t("settings-ruler-guide-vertical-description"),
    label: t("settings-ruler-guide-vertical"),
  },
  {
    action: "guideHorizontal",
    description: t("settings-ruler-guide-horizontal-description"),
    label: t("settings-ruler-guide-horizontal"),
  },
  {
    action: "cycleTolerance",
    description: t("settings-ruler-tolerance-description"),
    label: t("settings-ruler-tolerance"),
  },
  {
    action: "measureRadius",
    description: t("settings-ruler-radius-description"),
    label: t("settings-ruler-radius"),
  },
  { action: "toggleCenterlines", label: t("settings-ruler-centerlines") },
];

export function RulerSettingsPanel({
  activationDefault,
  defaults,
  isSaving,
  onActivationChange,
  onCaptureChange,
  onChange,
  savingShortcut,
  settings,
  shortcuts,
}: {
  isSaving: boolean;
  onActivationChange: (value: string | null) => void;
  onCaptureChange: (capturing: boolean) => Promise<void>;
  onChange: (settings: RulerSettings) => void;
  savingShortcut: boolean;
  settings: RulerSettings;
  shortcuts: ShortcutSettings | null;
  activationDefault?: string | null;
  defaults?: RulerSettings;
}) {
  const update = (changes: Partial<RulerSettings>) => {
    onChange({ ...settings, ...changes });
  };
  const isOff = isSaving || !settings.enabled;
  return (
    <div className="gap-layout flex flex-col">
      <GroupBox title={t("settings-section-ruler")}>
        <ControlRow
          description={t("settings-ruler-enabled-description")}
          title={t("settings-ruler-enabled")}
        >
          {(controlProps) => (
            <Switch
              {...controlProps}
              isDisabled={isSaving}
              isSelected={settings.enabled}
              onChange={(enabled) => {
                update({ enabled });
              }}
            />
          )}
        </ControlRow>
        <ControlRow title={t("settings-ruler-show")}>
          {(controlProps) => (
            <HotkeyField
              aria-describedby={controlProps["aria-describedby"]}
              aria-label={t("settings-ruler-show")}
              defaultValue={activationDefault}
              isDisabled={isOff || savingShortcut || !shortcuts}
              onCaptureChange={onCaptureChange}
              onChange={onActivationChange}
              value={
                shortcuts?.bindings.find(
                  (binding) => binding.action === "rulerOverlay",
                )?.shortcut ?? null
              }
            />
          )}
        </ControlRow>
      </GroupBox>
      <GroupBox title={t("settings-shortcuts")}>
        {actions().map(({ action, description, label }) => (
          <ControlRow description={description} key={action} title={label}>
            {(controlProps) => (
              <HotkeyField
                aria-describedby={controlProps["aria-describedby"]}
                aria-label={label}
                captureMode="local-shortcut"
                defaultValue={defaults?.bindings[action]}
                isDisabled={isOff}
                onCaptureChange={onCaptureChange}
                onChange={(shortcut) => {
                  update({
                    bindings: { ...settings.bindings, [action]: shortcut },
                  });
                }}
                value={settings.bindings[action] ?? null}
              />
            )}
          </ControlRow>
        ))}
      </GroupBox>
    </div>
  );
}
