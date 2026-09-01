// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Checkbox } from "../../components/base/checkbox/checkbox";
import { PillGroup } from "../../components/base/pill-group/pill-group";
import { Slider } from "../../components/base/slider/slider";

import { SettingRow } from "./setting-row";
import { GlideModifier, GlidePacing, GlideSettings } from "./types";

/** The four bare modifiers a gesture can be held with, in native notation. */
const isMac =
  typeof navigator !== "undefined" && navigator.userAgent.includes("Mac");
const modifiers: { id: GlideModifier; label: string; name: string }[] = isMac
  ? [
      { id: "command", label: "⌘", name: "Command" },
      { id: "option", label: "⌥", name: "Option" },
      { id: "control", label: "⌃", name: "Control" },
      { id: "shift", label: "⇧", name: "Shift" },
    ]
  : [
      { id: "command", label: "Win", name: "Windows" },
      { id: "option", label: "Alt", name: "Alt" },
      { id: "control", label: "Ctrl", name: "Control" },
      { id: "shift", label: "Shift", name: "Shift" },
    ];

const modifierItems = modifiers.map(({ id, label, name }) => ({
  ariaLabel: name,
  id,
  label,
}));

const pacings: { id: GlidePacing; label: string }[] = [
  { id: "snappy", label: "Snappy" },
  { id: "normal", label: "Normal" },
  { id: "relaxed", label: "Relaxed" },
];

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
          size="sm"
        />
      </SettingRow>
      <SettingRow
        description="Mouse users hold this while moving over a titlebar. Trackpads need no modifier - a two-finger scroll starts the glide."
        label="Mouse modifier"
      >
        <PillGroup
          ariaLabel="Mouse modifier"
          disabledIds={[settings.thirdsModifier]}
          display="label"
          isDisabled={isOff}
          items={modifierItems}
          onSelectionChange={(mouseModifier) => {
            update({ mouseModifier: mouseModifier as GlideModifier });
          }}
          selected={settings.mouseModifier}
        />
      </SettingRow>
      <SettingRow
        description="Held during a glide to target thirds instead of halves."
        label="Thirds modifier"
      >
        <PillGroup
          ariaLabel="Thirds modifier"
          disabledIds={[settings.mouseModifier]}
          display="label"
          isDisabled={isOff}
          items={modifierItems}
          onSelectionChange={(thirdsModifier) => {
            update({ thirdsModifier: thirdsModifier as GlideModifier });
          }}
          selected={settings.thirdsModifier}
        />
      </SettingRow>
      <SettingRow
        description="Breathing room around every window Glide places."
        label="Window gap"
      >
        <div className="w-44">
          <Slider
            aria-label="Window gap"
            isDisabled={isOff}
            maxValue={32}
            minValue={0}
            onChange={(windowGap) => {
              update({ windowGap });
            }}
            renderValue={(value) => `${String(value)} px`}
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
          size="sm"
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
            size="sm"
          />
        </SettingRow>
      )}
      <SettingRow
        description="How long a pause arms the next move."
        label="Gesture pacing"
      >
        <PillGroup
          ariaLabel="Gesture pacing"
          display="label"
          isDisabled={isOff}
          items={pacings}
          onSelectionChange={(pacing) => {
            update({ pacing: pacing as GlidePacing });
          }}
          selected={settings.pacing}
        />
      </SettingRow>
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
            size="sm"
          />
        </SettingRow>
      )}
    </div>
  );
}
