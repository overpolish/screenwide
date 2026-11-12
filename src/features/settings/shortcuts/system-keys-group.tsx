// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../composites/control-row/control-row";
import { platformArg, t } from "../../../i18n/i18n";
import { hotkeyKeys } from "../../../lib/hotkey";
import { Keyboard, Shortcut } from "../../../primitives/keyboard/keyboard";
import { Switch } from "../../../primitives/switch/switch";
import { SettingsGroup } from "../settings-group";

import type { ShortcutEntry } from "./shortcut-entry";

// The system's own screenshot keys, which can run Screenwide's tools in
// place of the system's: the switch, then each key the switch takes and the
// tool it opens. The keys are the system's, so they are shown, not set.
// Pointing at one rings it on the map, as the list's rows do.

export function SystemKeysGroup({
  entries,
  isOn,
  isSaving,
  onChange,
  onPoint,
}: {
  /** The keys taken while the switch is on. */
  entries: readonly ShortcutEntry[];
  isOn: boolean;
  isSaving: boolean;
  onChange: (isOn: boolean) => void;
  onPoint: (id: string | null) => void;
}) {
  return (
    <SettingsGroup title={t("settings-shortcuts-system-keys")}>
      <ControlRow
        description={t("settings-shortcuts-system-keys-replace-description")}
        title={t("settings-shortcuts-system-keys-replace", {
          platform: platformArg(),
        })}
      >
        {(controlProps) => (
          <Switch
            {...controlProps}
            isDisabled={isSaving}
            isSelected={isOn}
            onChange={onChange}
          />
        )}
      </ControlRow>
      {entries.map((entry) => (
        <div
          data-shortcut-id={entry.id}
          key={entry.id}
          onPointerEnter={() => {
            onPoint(entry.id);
          }}
          onPointerLeave={() => {
            onPoint(null);
          }}
        >
          <ControlRow title={entry.label}>
            {(controlProps) => (
              <Shortcut {...controlProps} role="group">
                {hotkeyKeys(entry.value).map((key) => (
                  <Keyboard key={key}>{key}</Keyboard>
                ))}
              </Shortcut>
            )}
          </ControlRow>
        </div>
      ))}
    </SettingsGroup>
  );
}
