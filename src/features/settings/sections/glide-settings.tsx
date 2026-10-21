// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../composites/control-row/control-row";
import { SliderNumberField } from "../../../composites/slider-number-field/slider-number-field";
import { t } from "../../../i18n/i18n";
import { isMacHost } from "../../../lib/platform";
import { Switch } from "../../../primitives/switch/switch";
import { SettingsGroup } from "../settings-group";

import type { GlideSettings } from "../../../bindings/GlideSettings";

const toggles = () =>
  [
    {
      key: "cursorFollows",
      macOnly: false,
      title: t("settings-glide-cursor-follows"),
    },
    {
      description: t("settings-glide-haptics-description"),
      key: "haptics",
      macOnly: true,
      title: t("settings-glide-haptics"),
    },
    {
      description: t("settings-glide-double-tap-description"),
      key: "doubleTapCenter",
      macOnly: false,
      title: t("settings-glide-double-tap"),
    },
  ] as const;

// Glide's held keys live with every other shortcut, in the Shortcuts pane.
export function GlideSettingsPanel({
  defaults,
  isSaving,
  onChange,
  settings,
}: {
  isSaving: boolean;
  onChange: (settings: GlideSettings) => void;
  settings: GlideSettings;
  /** What each setting returns to on reset. */
  defaults?: GlideSettings;
}) {
  const update = (changes: Partial<GlideSettings>) => {
    onChange({ ...settings, ...changes });
  };
  const isOff = isSaving || !settings.enabled;
  return (
    <SettingsGroup title={t("settings-section-glide")}>
      <ControlRow title={t("settings-glide-enabled")}>
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
      {toggles()
        .filter(({ macOnly }) => !macOnly || isMacHost)
        .map((toggle) => (
          <ControlRow
            description={
              "description" in toggle ? toggle.description : undefined
            }
            key={toggle.key}
            title={toggle.title}
          >
            {(controlProps) => (
              <Switch
                {...controlProps}
                isDisabled={isOff}
                isSelected={settings[toggle.key]}
                onChange={(selected) => {
                  update({ [toggle.key]: selected });
                }}
              />
            )}
          </ControlRow>
        ))}
      <ControlRow title={t("settings-glide-window-gap")}>
        {(controlProps) => (
          <div {...controlProps} role="group">
            <SliderNumberField
              aria-label={t("settings-glide-window-gap")}
              className="w-56"
              defaultValue={defaults?.windowGap}
              isDisabled={!settings.enabled}
              maxValue={32}
              minValue={0}
              onChange={(windowGap) => {
                update({ windowGap });
              }}
              unit="px"
              value={settings.windowGap}
            />
          </div>
        )}
      </ControlRow>
    </SettingsGroup>
  );
}
