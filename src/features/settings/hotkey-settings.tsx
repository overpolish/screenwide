// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { GroupBox } from "../../components/base/group-box/group-box";
import { HotkeyField } from "../../components/shared/hotkey-field/hotkey-field";
import { Setting } from "../../components/shared/setting/setting";

import type { ShortcutAction, ShortcutSettings } from "./types";

type ShortcutRow = {
  action: ShortcutAction;
  label: string;
  description?: string;
};

const groups: { rows: ShortcutRow[]; title: string }[] = [
  {
    rows: [
      {
        action: "toggleRecordingBar",
        label: "Show or hide recording controls",
      },
      {
        action: "startStopRecording",
        label: "Start or stop recording",
      },
      {
        action: "pauseResumeRecording",
        label: "Pause or resume recording",
      },
    ],
    title: "Recording",
  },
  {
    rows: [
      {
        action: "takeScreenshot",
        description: "Choose an area of your screen to capture.",
        label: "Take a screenshot",
      },
      {
        action: "takeScreenshotToClipboard",
        description: "Choose an area to copy, ready to paste elsewhere.",
        label: "Copy a screenshot",
      },
    ],
    title: "Screenshots",
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
      {groups.map((group) => (
        <GroupBox key={group.title} title={group.title}>
          {group.rows.map(({ action, description, label }) => (
            <Setting description={description} key={action} title={label}>
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
            </Setting>
          ))}
        </GroupBox>
      ))}
    </div>
  );
}
