// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { GroupBox } from "../../../components/base/group-box/group-box";
import { Switch } from "../../../components/base/switch/switch";
import { ControlRow } from "../../../components/shared/control-row/control-row";
import { HotkeyField } from "../../../components/shared/hotkey-field/hotkey-field";
import { t } from "../../../i18n/i18n";

import type { AnnotateSettings } from "../../../bindings/AnnotateSettings";

export function AnnotateSettingsPanel({
  activation,
  activationDefault,
  clear,
  clearDefault,
  isSaving,
  onActivationChange,
  onCaptureChange,
  onChange,
  onClearChange,
  settings,
}: {
  activation: string | null;
  clear: string | null;
  isSaving: boolean;
  onActivationChange: (value: string | null) => void;
  onCaptureChange: (capturing: boolean) => Promise<void>;
  onChange: (settings: AnnotateSettings) => void;
  onClearChange: (value: string | null) => void;
  settings: AnnotateSettings;
  activationDefault?: string | null;
  clearDefault?: string | null;
}) {
  const update = (changes: Partial<AnnotateSettings>) => {
    onChange({ ...settings, ...changes });
  };
  const isOff = isSaving || !settings.enabled;
  return (
    <div className="gap-layout flex flex-col">
      <GroupBox title={t("settings-section-annotate")}>
        <ControlRow
          description={t("settings-annotate-enabled-description")}
          title={t("settings-annotate-enabled")}
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
        <ControlRow title={t("settings-annotate-draw")}>
          {(controlProps) => (
            <HotkeyField
              aria-describedby={controlProps["aria-describedby"]}
              aria-label={t("settings-annotate-draw")}
              defaultValue={activationDefault}
              isDisabled={isOff}
              onCaptureChange={onCaptureChange}
              onChange={onActivationChange}
              value={activation}
            />
          )}
        </ControlRow>
        <ControlRow
          description={t("settings-annotate-keep-description")}
          title={t("settings-annotate-keep")}
        >
          {(controlProps) => (
            <Switch
              {...controlProps}
              isDisabled={isOff}
              isSelected={settings.keepAnnotationsBetweenSessions}
              onChange={(keepAnnotationsBetweenSessions) => {
                update({ keepAnnotationsBetweenSessions });
              }}
            />
          )}
        </ControlRow>
        <ControlRow title={t("settings-annotate-clear")}>
          {(controlProps) => (
            <HotkeyField
              aria-describedby={controlProps["aria-describedby"]}
              aria-label={t("settings-annotate-clear")}
              defaultValue={clearDefault}
              isDisabled={isOff}
              onCaptureChange={onCaptureChange}
              onChange={onClearChange}
              value={clear}
            />
          )}
        </ControlRow>
      </GroupBox>
    </div>
  );
}
