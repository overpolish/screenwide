// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../composites/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { Switch } from "../../../primitives/switch/switch";
import { HotkeyField } from "../hotkey-field/hotkey-field";
import { SettingsGroup } from "../settings-group";

import type { OcrAction } from "../../../bindings/OcrAction";
import type { OcrSettings } from "../../../bindings/OcrSettings";

const actions = (): {
  action: OcrAction;
  description: string;
  label: string;
}[] => [
  {
    action: "selectAll",
    description: t("settings-ocr-select-all-description"),
    label: t("settings-ocr-select-all"),
  },
  {
    action: "copyText",
    description: t("settings-ocr-copy-description"),
    label: t("settings-ocr-copy"),
  },
];

export function OcrSettingsPanel({
  activation,
  activationDefault,
  defaults,
  isSaving,
  onActivationChange,
  onCaptureChange,
  onChange,
  settings,
}: {
  activation: string | null;
  isSaving: boolean;
  onActivationChange: (value: string | null) => void;
  onCaptureChange: (capturing: boolean) => Promise<void>;
  onChange: (settings: OcrSettings) => void;
  settings: OcrSettings;
  activationDefault?: string | null;
  defaults?: OcrSettings;
}) {
  const update = (changes: Partial<OcrSettings>) => {
    onChange({ ...settings, ...changes });
  };
  const isOff = isSaving || !settings.enabled;
  return (
    <div className="gap-layout flex flex-col">
      <SettingsGroup title={t("settings-section-ocr")}>
        <ControlRow
          description={t("settings-ocr-enabled-description")}
          title={t("settings-ocr-enabled")}
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
        <ControlRow title={t("settings-ocr-activate")}>
          {(controlProps) => (
            <HotkeyField
              aria-describedby={controlProps["aria-describedby"]}
              aria-label={t("settings-ocr-activate")}
              defaultValue={activationDefault}
              isDisabled={isOff}
              onCaptureChange={onCaptureChange}
              onChange={onActivationChange}
              value={activation}
            />
          )}
        </ControlRow>
      </SettingsGroup>
      <SettingsGroup title={t("settings-shortcuts")}>
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
      </SettingsGroup>
    </div>
  );
}
