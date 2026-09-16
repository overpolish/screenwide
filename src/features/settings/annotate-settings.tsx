// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { GroupBox } from "../../components/base/group-box/group-box";
import { Switch } from "../../components/base/switch/switch";
import { HotkeyField } from "../../components/shared/hotkey-field/hotkey-field";
import { Setting } from "../../components/shared/setting/setting";

import type { AnnotateSettings } from "./types";

export function AnnotateSettingsPanel({
  activation,
  activationDefault,
  clear,
  clearDefault,
  isSaving,
  onActivationChange,
  onCaptureChange,
  onChange,
  onClearChange,
  settings,
}: {
  activation: string | null;
  clear: string | null;
  isSaving: boolean;
  onActivationChange: (value: string | null) => void;
  onCaptureChange: (capturing: boolean) => Promise<void>;
  onChange: (settings: AnnotateSettings) => void;
  onClearChange: (value: string | null) => void;
  settings: AnnotateSettings;
  activationDefault?: string | null;
  clearDefault?: string | null;
}) {
  const update = (changes: Partial<AnnotateSettings>) => {
    onChange({ ...settings, ...changes });
  };
  const isOff = isSaving || !settings.enabled;
  return (
    <div className="gap-layout flex flex-col">
      <GroupBox title="Annotate">
        <Setting
          description="Draw on your screen, and keep the annotations in a recording."
          title="Use Annotate"
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
        <Setting title="Draw on the screen">
          {(controlProps) => (
            <HotkeyField
              aria-describedby={controlProps["aria-describedby"]}
              aria-label="Draw on the screen"
              defaultValue={activationDefault}
              isDisabled={isOff}
              onCaptureChange={onCaptureChange}
              onChange={onActivationChange}
              value={activation}
            />
          )}
        </Setting>
        <Setting
          description="They stay on screen until you clear them."
          title="Keep annotations after exiting"
        >
          {(controlProps) => (
            <Switch
              {...controlProps}
              isDisabled={isOff}
              isSelected={settings.keepAnnotationsBetweenSessions}
              onChange={(keepAnnotationsBetweenSessions) => {
                update({ keepAnnotationsBetweenSessions });
              }}
            />
          )}
        </Setting>
        <Setting title="Clear annotations">
          {(controlProps) => (
            <HotkeyField
              aria-describedby={controlProps["aria-describedby"]}
              aria-label="Clear annotations"
              defaultValue={clearDefault}
              isDisabled={isOff}
              onCaptureChange={onCaptureChange}
              onChange={onClearChange}
              value={clear}
            />
          )}
        </Setting>
      </GroupBox>
    </div>
  );
}
