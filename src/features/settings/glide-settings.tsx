// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { GroupBox } from "../../components/base/group-box/group-box";
import { Switch } from "../../components/base/switch/switch";
import { HotkeyField } from "../../components/shared/hotkey-field/hotkey-field";
import { Setting } from "../../components/shared/setting/setting";
import { SliderNumberField } from "../../components/shared/slider-number-field/slider-number-field";

import type { GlideSettings } from "./types";

const isMac =
  typeof navigator !== "undefined" && navigator.userAgent.includes("Mac");
const toggles = [
  {
    key: "cursorFollows",
    macOnly: false,
    title: "Move pointer with window",
  },
  {
    description: "Feel a tap when the next move is ready.",
    key: "haptics",
    macOnly: true,
    title: "Trackpad feedback",
  },
  {
    description: "Two fingers, or hold mouse control while double-clicking.",
    key: "doubleTapCenter",
    macOnly: false,
    title: "Double-tap or double-click to center",
  },
] as const;

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
      <GroupBox title="Glide">
        <Setting
          description="Move and resize windows to fit your screen."
          title="Use Glide"
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
        </Setting>
      </GroupBox>
      <GroupBox title="Controls">
        {(
          [
            "mouseModifier",
            "thirdsModifier",
            "spacesModifier",
            "monitorsModifier",
          ] as const
        ).map((key) => {
          const otherKeys = (
            [
              "mouseModifier",
              "thirdsModifier",
              "spacesModifier",
              "monitorsModifier",
            ] as const
          ).filter((candidate) => candidate !== key);
          const title =
            key === "mouseModifier"
              ? "Glide control for mouse"
              : key === "thirdsModifier"
                ? "Control for screen thirds"
                : key === "spacesModifier"
                  ? isMac
                    ? "Modifier for Spaces"
                    : "Modifier for virtual desktops"
                  : "Modifier for Monitors";
          return (
            <Setting
              description={
                key === "mouseModifier"
                  ? "Hold while moving from a window's top bar."
                  : key === "thirdsModifier"
                    ? "Hold during Glide to use thirds instead of halves."
                    : key === "spacesModifier"
                      ? isMac
                        ? "Hold while moving between Spaces."
                        : "Hold while moving between virtual desktops."
                      : "Hold while moving between monitors."
              }
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
            </Setting>
          );
        })}
      </GroupBox>
      <GroupBox title="Behavior">
        <Setting title="Space around windows">
          {(controlProps) => (
            <div {...controlProps} role="group">
              <SliderNumberField
                aria-label="Space around windows"
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
        </Setting>
        {toggles
          .filter(({ macOnly }) => !macOnly || isMac)
          .map((toggle) => (
            <Setting
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
            </Setting>
          ))}
      </GroupBox>
    </div>
  );
}
