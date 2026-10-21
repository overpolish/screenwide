// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../composites/control-row/control-row";
import { platformArg, t } from "../../../i18n/i18n";
import { HotkeyField } from "../hotkey-field/hotkey-field";

import type { GlideSettings } from "../../../bindings/GlideSettings";

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

// The keys held while dragging a window to steer Glide, one each. Taking a
// key another one already has swaps the two, so no two ever share a key.
export function GlideKeys({
  defaults,
  isDisabled,
  onCaptureChange,
  onChange,
  settings,
}: {
  isDisabled: boolean;
  onCaptureChange: (capturing: boolean) => Promise<void>;
  onChange: (settings: GlideSettings) => void;
  settings: GlideSettings;
  defaults?: GlideSettings;
}) {
  const update = (changes: Partial<GlideSettings>) => {
    onChange({ ...settings, ...changes });
  };
  return (
    <>
      {modifierKeys.map((key) => {
        const otherKeys = modifierKeys.filter((candidate) => candidate !== key);
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
                isDisabled={isDisabled}
                onCaptureChange={onCaptureChange}
                onChange={(value) => {
                  if (value === null) return;
                  const collidingKey = otherKeys.find(
                    (candidate) => settings[candidate] === value,
                  );
                  update({
                    [key]: value,
                    ...(collidingKey ? { [collidingKey]: settings[key] } : {}),
                  });
                }}
                value={settings[key]}
              />
            )}
          </ControlRow>
        );
      })}
    </>
  );
}
