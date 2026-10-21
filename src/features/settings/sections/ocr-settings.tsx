// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../composites/control-row/control-row";
import { t } from "../../../i18n/i18n";
import { Switch } from "../../../primitives/switch/switch";
import { SettingsGroup } from "../settings-group";

import type { OcrSettings } from "../../../bindings/OcrSettings";

// OCR's keys live with every other shortcut, in the Shortcuts pane.
export function OcrSettingsPanel({
  isSaving,
  onChange,
  settings,
}: {
  isSaving: boolean;
  onChange: (settings: OcrSettings) => void;
  settings: OcrSettings;
}) {
  return (
    <SettingsGroup title={t("settings-section-ocr")}>
      <ControlRow title={t("settings-ocr-enabled")}>
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
