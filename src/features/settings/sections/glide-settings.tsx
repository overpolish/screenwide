// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../composites/control-row/control-row";
import { HotkeyField } from "../../../composites/hotkey-field/hotkey-field";
import { SliderNumberField } from "../../../composites/slider-number-field/slider-number-field";
import { platformArg, t } from "../../../i18n/i18n";
import { isMacHost } from "../../../lib/platform";
import { GroupBox } from "../../../primitives/group-box/group-box";
import { Switch } from "../../../primitives/switch/switch";

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

const modifierTitle = (key: ModifierKey) => {
  switch (key) {
    case "mouseModifier":
      return t("settings-glide-mouse-modifier");
    case "thirdsModifier":
      return t("settings-glide-thirds-modifier");
    case "spacesModifier":
      return t("settings-glide-spaces-modifier", { platform: platformArg() });
    case "monitorsModifier":
      return t("settings-glide-monitors-modifier");
  }
};

const modifierDescription = (key: ModifierKey) => {
  switch (key) {
    case "mouseModifier":
      return t("settings-glide-mouse-modifier-description");
    case "thirdsModifier":
      return t("settings-glide-thirds-modifier-description");
    case "spacesModifier":
      return t("settings-glide-spaces-modifier-description", {
        platform: platformArg(),
      });
    case "monitorsModifier":
      return t("settings-glide-monitors-modifier-description");
  }
};

const modifierKeys = [
  "mouseModifier",
  "thirdsModifier",
  "spacesModifier",
  "monitorsModifier",
] as const;

type ModifierKey = (typeof modifierKeys)[number];

export function GlideSettingsPanel({
  defaults,
  isSaving,
  onCaptureChange,
  onChange,
  settings,
}: {
  isSaving: boolean;
  onCaptureChange: (capturing: boolean) => Promise<void>;
  onChange: (settings: GlideSettings) => void;
  settings: GlideSettings;
  defaults?: GlideSettings;
}) {
  const update = (changes: Partial<GlideSettings>) => {
    onChange({ ...settings, ...changes });
  };
  const isOff = isSaving || !settings.enabled;
  return (
    <div className="gap-layout flex flex-col">
      <GroupBox title={t("settings-section-glide")}>
        <ControlRow
          description={t("settings-glide-enabled-description")}
          title={t("settings-glide-enabled")}
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
      </GroupBox>
      <GroupBox title={t("settings-glide-controls")}>
        {modifierKeys.map((key) => {
          const otherKeys = modifierKeys.filter(
            (candidate) => candidate !== key,
          );
          const title = modifierTitle(key);
          return (
            <ControlRow
              description={modifierDescription(key)}
              key={key}
              title={title}
            >
              {(controlProps) => (
                <HotkeyField
                  aria-describedby={controlProps["aria-describedby"]}
                  aria-label={title}
                  captureMode="single-control"
                  defaultValue={defaults?.[key]}
                  isClearable={false}
                  isDisabled={isOff}
                  onCaptureChange={onCaptureChange}
                  onChange={(value) => {
                    if (value === null) return;
                    const collidingKey = otherKeys.find(
                      (candidate) => settings[candidate] === value,
                    );
                    update({
                      [key]: value,
                      ...(collidingKey
                        ? { [collidingKey]: settings[key] }
                        : {}),
                    });
                  }}
                  value={settings[key]}
                />
              )}
            </ControlRow>
          );
        })}
      </GroupBox>
      <GroupBox title={t("settings-glide-behavior")}>
        <ControlRow title={t("settings-glide-window-gap")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <SliderNumberField
                aria-label={t("settings-glide-window-gap")}
                className="w-56"
                isDisabled={!settings.enabled}
                maxValue={32}
                minValue={0}
                onChange={(windowGap) => {
                  update({ windowGap });
                }}
                rightSection="px"
                value={settings.windowGap}
              />
            </div>
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
      </GroupBox>
    </div>
  );
}
