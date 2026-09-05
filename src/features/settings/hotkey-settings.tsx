// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { HotkeyField } from "../../components/shared/hotkey-field/hotkey-field";
import { Setting } from "../../components/shared/setting/setting";

import type { ShortcutAction, ShortcutSettings } from "./types";

const actions: {
  action: ShortcutAction;
  label: string;
  description?: string;
}[] = [
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
  {
    action: "recognizeText",
    description: "Select text or a QR code on your screen.",
    label: "Read text or a QR code",
  },
  {
    action: "rulerOverlay",
    description: "Measure sizes and distances on your screen.",
    label: "Show ruler",
  },
];

export function HotkeySettingsPanel({
  onCaptureChange,
  onChange,
  saving,
  settings,
}: {
  onCaptureChange: (capturing: boolean) => Promise<void>;
  onChange: (action: ShortcutAction, shortcut: string | null) => void;
  saving: ShortcutAction | null;
  settings: ShortcutSettings | null;
}) {
  return (
    <div className="gap-layout flex flex-col">
      {actions.map(({ action, description, label }) => (
        <Setting description={description} key={action} title={label}>
          {(controlProps) => (
            <HotkeyField
              aria-describedby={controlProps["aria-describedby"]}
              aria-label={label}
              isDisabled={!settings || saving !== null}
              onCaptureChange={onCaptureChange}
              onChange={(shortcut) => {
                onChange(action, shortcut);
              }}
              value={
                settings?.bindings.find((binding) => binding.action === action)
                  ?.shortcut ?? null
              }
            />
          )}
        </Setting>
      ))}
    </div>
  );
}
