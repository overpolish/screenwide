// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { GroupBox } from "../../../components/base/group-box/group-box";
import { Switch } from "../../../components/base/switch/switch";
import { ControlRow } from "../../../components/shared/control-row/control-row";
import { HotkeyField } from "../../../components/shared/hotkey-field/hotkey-field";

import type { OcrAction } from "../../../bindings/OcrAction";
import type { OcrSettings } from "../../../bindings/OcrSettings";

const actions: { action: OcrAction; description: string; label: string }[] = [
  {
    action: "selectAll",
    description: "Select all recognized text.",
    label: "Select all text",
  },
  {
    action: "copyText",
    description: "Copy selected text, or everything when nothing is selected.",
    label: "Copy text",
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
      <GroupBox title="OCR">
        <ControlRow
          description="Select text or a QR code on your screen."
          title="Use OCR"
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
        <ControlRow title="Read text or a QR code">
          {(controlProps) => (
            <HotkeyField
              aria-describedby={controlProps["aria-describedby"]}
              aria-label="Read text or a QR code"
              defaultValue={activationDefault}
              isDisabled={isOff}
              onCaptureChange={onCaptureChange}
              onChange={onActivationChange}
              value={activation}
            />
          )}
        </ControlRow>
      </GroupBox>
      <GroupBox title="Shortcuts">
        {actions.map(({ action, description, label }) => (
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
