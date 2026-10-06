// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { GroupBox } from "../../../components/base/group-box/group-box";
import { PillGroup } from "../../../components/base/pill-group/pill-group";
import { Switch } from "../../../components/base/switch/switch";
import { ControlRow } from "../../../components/shared/control-row/control-row";
import { PathField } from "../../../components/shared/path-field/path-field";
import { useRecordingInputStore } from "../../recording-inputs/store";
import { RecordingFps } from "../../recording-inputs/types";
import { useSettingsApi } from "../settings-api-context";
import { LiveSoftwareUpdateSetting } from "../settings-update-actions";

import type { GeneralSettings } from "../types";

/** The preferences a Switch row can drive. */
type ToggleKey = {
  [K in keyof GeneralSettings]: GeneralSettings[K] extends boolean ? K : never;
}[keyof GeneralSettings];

type ToggleItem = { key: ToggleKey; title: string; description?: string };

const recordingToggle = {
  description: "Check your camera, microphone and computer sound are working.",
  key: "showRecordingConfidenceChecks",
  title: "Show camera and sound while recording",
} satisfies ToggleItem;

const autoZoomToggle = {
  description: "Zoom in where you click and type.",
  key: "autoZoom",
  title: "Zoom in automatically",
} satisfies ToggleItem;

const captureToggle = {
  description: "Show Screenwide's own windows in recordings and screenshots.",
  key: "recordScreenwideWindows",
  title: "Include Screenwide in captures",
} satisfies ToggleItem;

const startupToggles: ToggleItem[] = [
  { key: "launchAtLogin", title: "Start when you sign in" },
  {
    key: "showRecordingBarOnLaunch",
    title: "Show recording controls on startup",
  },
];

/** The folders Settings names: where projects live, and where exports go. */
const folders = [
  {
    description: "New recordings and their edits are kept here.",
    key: "projectDirectory",
    kind: "project",
    title: "Projects folder",
  },
  {
    description: "You can choose a different folder when saving.",
    key: "recordingDirectory",
    kind: "recording",
    title: "Recording folder",
  },
  {
    description: "You can choose a different folder when saving.",
    key: "screenshotDirectory",
    kind: "screenshot",
    title: "Screenshot folder",
  },
] as const;

export function GeneralSettingsPanel({
  isSaving,
  onChange,
  onError,
  settings,
  updateSetting = <LiveSoftwareUpdateSetting />,
}: {
  isSaving: boolean;
  onChange: (settings: GeneralSettings) => void;
  onError: (message: string) => void;
  settings: GeneralSettings;
  updateSetting?: ReactNode;
}) {
  const { browseDefaultLocation } = useSettingsApi();
  // The frame rate lives with the recording inputs the bar records with, not
  // in the stored preferences; settings only offers the choice.
  const fps = useRecordingInputStore((state) => state.fps);
  const setFps = useRecordingInputStore((state) => state.setFps);
  const update = (changes: Partial<GeneralSettings>) => {
    onChange({ ...settings, ...changes });
  };
  const toggle = (item: ToggleItem) => (
    <ControlRow
      description={item.description}
      key={item.key}
      title={item.title}
    >
      {(controlProps) => (
        <Switch
          {...controlProps}
          isDisabled={isSaving}
          isSelected={settings[item.key]}
          onChange={(selected) => {
            update({ [item.key]: selected });
          }}
        />
      )}
    </ControlRow>
  );
  return (
    <div className="gap-layout flex flex-col">
      <GroupBox title="Software Update">{updateSetting}</GroupBox>
      <GroupBox title="Appearance">
        <ControlRow
          description="Controls and highlights use the system accent colour, or Screenwide's own."
          title="Accent colour"
        >
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label="Accent colour"
                display="label"
                isDisabled={isSaving}
                items={[
                  { id: "system", label: "System" },
                  { id: "screenwide", label: "Screenwide" },
                ]}
                onSelectionChange={(accent) => {
                  update({ accent: accent as GeneralSettings["accent"] });
                }}
                selected={settings.accent}
              />
            </div>
          )}
        </ControlRow>
      </GroupBox>
      <GroupBox title="Saving">
        {folders.map(({ description, key, kind, title }) => (
          <ControlRow description={description} key={kind} title={title}>
            {(controlProps) => (
              <div {...controlProps} role="group">
                <PathField
                  aria-label={title}
                  emptyLabel="Default folder"
                  isDisabled={isSaving}
                  onBrowse={() => {
                    void browseDefaultLocation(kind)
                      .then((directory) => {
                        if (directory) update({ [key]: directory });
                      })
                      .catch((reason: unknown) => {
                        onError(String(reason));
                      });
                  }}
                  secondaryAction={{
                    label: `Use the default ${kind} folder`,
                    onPress: () => {
                      update({ [key]: null });
                    },
                    type: "reset",
                  }}
                  value={settings[key]}
                />
              </div>
            )}
          </ControlRow>
        ))}
      </GroupBox>
      <GroupBox title="Recording">
        {toggle(recordingToggle)}
        {toggle(autoZoomToggle)}
        <ControlRow title="Recording countdown">
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label="Recording countdown"
                display="label"
                isDisabled={isSaving}
                items={[
                  { id: "0", label: "Off" },
                  { id: "3", label: "3s" },
                  { id: "5", label: "5s" },
                ]}
                onSelectionChange={(seconds) => {
                  update({
                    recordingCountdownSeconds: Number(seconds) as 0 | 3 | 5,
                  });
                }}
                selected={String(settings.recordingCountdownSeconds)}
              />
            </div>
          )}
        </ControlRow>
        <ControlRow
          description="More frames look smoother and make a larger file."
          title="Frame rate"
        >
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label="Frame rate"
                display="label"
                items={[
                  { id: "30", label: "30 fps" },
                  { id: "60", label: "60 fps" },
                ]}
                onSelectionChange={(selected) => {
                  setFps(Number(selected) as RecordingFps);
                }}
                selected={String(fps)}
              />
            </div>
          )}
        </ControlRow>
      </GroupBox>
      <GroupBox title="Capture">
        {toggle(captureToggle)}
        <ControlRow title="Delayed Screenshot">
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label="Delayed Screenshot"
                display="label"
                isDisabled={isSaving}
                items={[
                  { id: "3", label: "3s" },
                  { id: "5", label: "5s" },
                  { id: "10", label: "10s" },
                ]}
                onSelectionChange={(seconds) => {
                  update({
                    screenshotDelaySeconds: Number(seconds) as 3 | 5 | 10,
                  });
                }}
                selected={String(settings.screenshotDelaySeconds)}
              />
            </div>
          )}
        </ControlRow>
      </GroupBox>
      <GroupBox title="Startup">
        {startupToggles.map((item) => toggle(item))}
      </GroupBox>
    </div>
  );
}
