// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { GroupBox } from "../../../components/base/group-box/group-box";
import { ControlRow } from "../../../components/shared/control-row/control-row";
import { HotkeyField } from "../../../components/shared/hotkey-field/hotkey-field";
import { t } from "../../../i18n/i18n";

import type { ShortcutAction } from "../../../bindings/ShortcutAction";
import type { ShortcutSettings } from "../../../bindings/ShortcutSettings";

type ShortcutRow = {
  action: ShortcutAction;
  label: string;
  description?: string;
};

const groups = (): { id: string; rows: ShortcutRow[]; title: string }[] => [
  {
    id: "recording",
    rows: [
      {
        action: "toggleRecordingBar",
        label: t("settings-shortcut-toggle-bar"),
      },
      {
        action: "startStopRecording",
        label: t("settings-shortcut-start-stop"),
      },
      {
        action: "pauseResumeRecording",
        label: t("settings-shortcut-pause-resume"),
      },
      {
        action: "saveReplay",
        description: t("settings-shortcut-save-replay-description"),
        label: t("settings-shortcut-save-replay"),
      },
    ],
    title: t("settings-recording"),
  },
  {
    id: "screenshots",
    rows: [
      {
        action: "takeScreenshot",
        description: t("settings-shortcut-screenshot-description"),
        label: t("settings-shortcut-screenshot"),
      },
      {
        action: "takeScreenshotToClipboard",
        description: t("settings-shortcut-copy-screenshot-description"),
        label: t("settings-shortcut-copy-screenshot"),
      },
    ],
    title: t("settings-screenshots"),
  },
];

export function HotkeySettingsPanel({
  defaults,
  onCaptureChange,
  onChange,
  saving,
  settings,
}: {
  onCaptureChange: (capturing: boolean) => Promise<void>;
  onChange: (action: ShortcutAction, shortcut: string | null) => void;
  saving: ShortcutAction | null;
  settings: ShortcutSettings | null;
  defaults?: ShortcutSettings | null;
}) {
  return (
    <div className="gap-layout flex flex-col">
      {groups().map((group) => (
        <GroupBox key={group.id} title={group.title}>
          {group.rows.map(({ action, description, label }) => (
            <ControlRow description={description} key={action} title={label}>
              {(controlProps) => (
                <HotkeyField
                  aria-describedby={controlProps["aria-describedby"]}
                  aria-label={label}
                  defaultValue={
                    defaults?.bindings.find(
                      (binding) => binding.action === action,
                    )?.shortcut
                  }
                  isDisabled={!settings || saving !== null}
                  onCaptureChange={onCaptureChange}
                  onChange={(shortcut) => {
                    onChange(action, shortcut);
                  }}
                  value={
                    settings?.bindings.find(
                      (binding) => binding.action === action,
                    )?.shortcut ?? null
                  }
                />
              )}
            </ControlRow>
          ))}
        </GroupBox>
      ))}
    </div>
  );
}
