// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { GroupBox } from "../../components/base/group-box/group-box";
import { Switch } from "../../components/base/switch/switch";
import { HotkeyField } from "../../components/shared/hotkey-field/hotkey-field";
import { Setting } from "../../components/shared/setting/setting";

import type { RulerAction, RulerSettings, ShortcutSettings } from "./types";

const actions: {
  action: RulerAction;
  label: string;
  description?: string;
}[] = [
  { action: "toggleCrosshair", label: "Toggle crosshair" },
  {
    action: "copyColour",
    description: "Copies the colour under the pointer as a hex code.",
    label: "Copy colour",
  },
  {
    action: "deleteMeasurement",
    description:
      "Hover a label or line; otherwise deletes the latest measurement.",
    label: "Delete measurement",
  },
  {
    action: "copyMeasurement",
    description: "Copies the latest measurement, regardless of what you hover.",
    label: "Copy measurement",
  },
  { action: "undo", label: "Undo" },
  { action: "redo", label: "Redo" },
  {
    action: "stampHorizontal",
    description: "Press to stamp; or hold, move, and release.",
    label: "Horizontal stamp",
  },
  {
    action: "stampVertical",
    description: "Press to stamp; or hold, move, and release.",
    label: "Vertical stamp",
  },
  {
    action: "guideVertical",
    description: "Hold and click to place vertical guides.",
    label: "Vertical guide",
  },
  {
    action: "guideHorizontal",
    description: "Hold and click to place horizontal guides.",
    label: "Horizontal guide",
  },
  {
    action: "cycleTolerance",
    description: "Switch between clear, balanced, and subtle edge detection.",
    label: "Cycle tolerance",
  },
  {
    action: "measureRadius",
    description: "Hold over a corner to measure its radius.",
    label: "Measure radius",
  },
  { action: "toggleCenterlines", label: "Toggle centerlines" },
];

export function RulerSettingsPanel({
  activationDefault,
  defaults,
  isSaving,
  onActivationChange,
  onCaptureChange,
  onChange,
  savingShortcut,
  settings,
  shortcuts,
}: {
  isSaving: boolean;
  onActivationChange: (value: string | null) => void;
  onCaptureChange: (capturing: boolean) => Promise<void>;
  onChange: (settings: RulerSettings) => void;
  savingShortcut: boolean;
  settings: RulerSettings;
  shortcuts: ShortcutSettings | null;
  activationDefault?: string | null;
  defaults?: RulerSettings;
}) {
  const update = (changes: Partial<RulerSettings>) => {
    onChange({ ...settings, ...changes });
  };
  const isOff = isSaving || !settings.enabled;
  return (
    <div className="gap-layout flex flex-col">
      <GroupBox title="Ruler">
        <Setting
          description="Measure sizes and distances on your screen."
          title="Use Ruler"
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
        <Setting title="Show ruler">
          {(controlProps) => (
            <HotkeyField
              aria-describedby={controlProps["aria-describedby"]}
              aria-label="Show ruler"
              defaultValue={activationDefault}
              isDisabled={isOff || savingShortcut || !shortcuts}
              onCaptureChange={onCaptureChange}
              onChange={onActivationChange}
              value={
                shortcuts?.bindings.find(
                  (binding) => binding.action === "rulerOverlay",
                )?.shortcut ?? null
              }
            />
          )}
        </Setting>
      </GroupBox>
      <GroupBox title="Shortcuts">
        {actions.map(({ action, description, label }) => (
          <Setting description={description} key={action} title={label}>
            {(controlProps) => (
              <HotkeyField
                aria-describedby={controlProps["aria-describedby"]}
                aria-label={label}
                captureMode="local-shortcut"
                defaultValue={defaults?.bindings[action]}
                isDisabled={isOff}
                onCaptureChange={onCaptureChange}
                onChange={(shortcut) => {
                  update({
                    bindings: { ...settings.bindings, [action]: shortcut },
                  });
                }}
                value={settings.bindings[action]}
              />
            )}
          </Setting>
        ))}
      </GroupBox>
    </div>
  );
}
