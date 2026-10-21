// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../composites/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { Switch } from "../../../primitives/switch/switch";
import { SettingsGroup } from "../settings-group";

import type { RulerSettings } from "../../../bindings/RulerSettings";

// The ruler's keys live with every other shortcut, in the Shortcuts pane.
export function RulerSettingsPanel({
  isSaving,
  onChange,
  settings,
}: {
  isSaving: boolean;
  onChange: (settings: RulerSettings) => void;
  settings: RulerSettings;
}) {
  return (
    <SettingsGroup title={t("settings-section-ruler")}>
      <ControlRow title={t("settings-ruler-enabled")}>
        {(controlProps) => (
          <Switch
            {...controlProps}
            isDisabled={isSaving}
            isSelected={settings.enabled}
            onChange={(enabled) => {
              onChange({ ...settings, enabled });
            }}
          />
        )}
      </ControlRow>
    </SettingsGroup>
  );
}
