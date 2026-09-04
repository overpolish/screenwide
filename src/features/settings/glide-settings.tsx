// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Checkbox } from "../../components/base/checkbox/checkbox";
import { Slider } from "../../components/base/slider/slider";

import { KeyField } from "./key-field";
import { SettingRow } from "./setting-row";
import { GlideSettings } from "./types";

const isMac =
  typeof navigator !== "undefined" && navigator.userAgent.includes("Mac");

export function GlideSettingsPanel({
  isSaving,
  onChange,
  settings,
}: {
  isSaving: boolean;
  onChange: (settings: GlideSettings) => void;
  settings: GlideSettings;
}) {
  const update = (changes: Partial<GlideSettings>) => {
    onChange({ ...settings, ...changes });
  };
  // The master toggle stays live while it is off; everything it governs reads
  // as unavailable, which the native side already treats as inert.
  const isOff = isSaving || !settings.enabled;

  return (
    <div className="divide-y divide-muted/15 px-4">
      <SettingRow
        description="Glide windows into halves, quarters and thirds - a two-finger scroll on a trackpad, or the mouse with its modifier held."
        label="Enable Glide"
      >
        <Checkbox
          isDisabled={isSaving}
          isSelected={settings.enabled}
          onChange={(enabled) => {
            update({ enabled });
          }}
        />
      </SettingRow>
      <SettingRow
        description="Mouse users hold this key or auxiliary button while moving over a titlebar. Trackpads need no control - a two-finger gesture starts the glide."
        label="Mouse control"
      >
        <KeyField
          ariaLabel="Mouse activation control"
          isDisabled={isOff}
          onChange={(mouseModifier) => {
            update(
              mouseModifier === settings.thirdsModifier
                ? { mouseModifier, thirdsModifier: settings.mouseModifier }
                : { mouseModifier },
            );
          }}
          value={settings.mouseModifier}
        />
      </SettingRow>
      <SettingRow
        description="A key or auxiliary mouse button held during a glide to target thirds instead of halves."
        label="Thirds control"
      >
        <KeyField
          ariaLabel="Thirds control"
          isDisabled={isOff}
          onChange={(thirdsModifier) => {
            update(
              thirdsModifier === settings.mouseModifier
                ? { mouseModifier: settings.thirdsModifier, thirdsModifier }
                : { thirdsModifier },
            );
          }}
          value={settings.thirdsModifier}
        />
      </SettingRow>
      <SettingRow
        description="Breathing room around every window Glide places."
        label="Window gap"
      >
        <div className="gap-control flex w-44 flex-col">
          <span className="text-right text-xs text-muted tabular-nums">
            {settings.windowGap.toString()} px
          </span>
          <Slider
            aria-label="Window gap"
            isDisabled={isOff}
            maxValue={32}
            minValue={0}
            onChange={(windowGap) => {
              update({ windowGap });
            }}
            step={1}
            value={settings.windowGap}
          />
        </div>
      </SettingRow>
      <SettingRow
        description="After a move, the cursor lands on the window it carried."
        label="Cursor follows window"
      >
        <Checkbox
          isDisabled={isOff}
          isSelected={settings.cursorFollows}
          onChange={(cursorFollows) => {
            update({ cursorFollows });
          }}
        />
      </SettingRow>
      {isMac && (
        <SettingRow
          description="A tick from the trackpad when the next move is ready."
          label="Haptic feedback"
        >
          <Checkbox
            isDisabled={isOff}
            isSelected={settings.haptics}
            onChange={(haptics) => {
              update({ haptics });
            }}
          />
        </SettingRow>
      )}
      {isMac && (
        <SettingRow
          description="Two quick taps on a titlebar - or a modifier double-click - center the window."
          label="Double-tap to center"
        >
          <Checkbox
            isDisabled={isOff}
            isSelected={settings.doubleTapCenter}
            onChange={(doubleTapCenter) => {
              update({ doubleTapCenter });
            }}
          />
        </SettingRow>
      )}
    </div>
  );
}
