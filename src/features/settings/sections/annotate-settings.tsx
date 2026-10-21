// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../composites/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { Switch } from "../../../primitives/switch/switch";
import { SettingsGroup } from "../settings-group";

import type { AnnotateSettings } from "../../../bindings/AnnotateSettings";

// Annotate's keys live with every other shortcut, in the Shortcuts pane.
export function AnnotateSettingsPanel({
  isSaving,
  onChange,
  settings,
}: {
  isSaving: boolean;
  onChange: (settings: AnnotateSettings) => void;
  settings: AnnotateSettings;
}) {
  const update = (changes: Partial<AnnotateSettings>) => {
    onChange({ ...settings, ...changes });
  };
  return (
    <SettingsGroup title={t("settings-section-annotate")}>
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
      <ControlRow
        description={t("settings-annotate-keep-description")}
        title={t("settings-annotate-keep")}
      >
        {(controlProps) => (
          <Switch
            {...controlProps}
            isDisabled={isSaving || !settings.enabled}
            isSelected={settings.keepAnnotationsBetweenSessions}
            onChange={(keepAnnotationsBetweenSessions) => {
              update({ keepAnnotationsBetweenSessions });
            }}
          />
        )}
      </ControlRow>
    </SettingsGroup>
  );
}
