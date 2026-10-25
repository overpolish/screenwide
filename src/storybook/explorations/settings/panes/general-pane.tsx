// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { PathField } from "../../../../composites/path-field/path-field";
import { useSettingsApi } from "../../../../features/settings/settings-api-context";
import { SettingsGroup } from "../../../../features/settings/settings-group";
import { SoftwareUpdateSetting } from "../../../../features/settings/settings-update-actions";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { Switch } from "../../../../primitives/switch/switch";
import { CaptureArt, StartupArt } from "../preview/general-art";

import { Pane } from "./pane";

import type { GeneralSettings } from "../../../../bindings/GeneralSettings";
import type { ExploredSettings } from "../settings-state";

// General: the settings about the app itself, now that Recording has its
// own pane. A row whose value is hard to picture carries a small picture of
// it at its start, which changes with the switch.

const FOLDERS = [
  {
    description: "New recordings and their edits are kept here.",
    key: "projectDirectory",
    kind: "project",
    title: "Projects folder",
  },
  {
    description: "Where exports go. You can choose another when saving.",
    key: "recordingDirectory",
    kind: "recording",
    title: "Recording folder",
  },
  {
    description: "Where screenshots go.",
    key: "screenshotDirectory",
    kind: "screenshot",
    title: "Screenshot folder",
  },
] as const;

export function GeneralPane({ settings }: { settings: ExploredSettings }) {
  const { browseDefaultLocation } = useSettingsApi();
  const [checked, setChecked] = useState(false);
  const { general, setGeneral } = settings;
  if (!general) return null;
  const update = (changes: Partial<GeneralSettings>) => {
    setGeneral({ ...general, ...changes });
  };

  return (
    <Pane label="General">
      <SettingsGroup title="Saving">
        {FOLDERS.map(({ description, key, kind, title }) => (
          <ControlRow description={description} key={kind} title={title}>
            {(controlProps) => (
              <div {...controlProps} role="group">
                <PathField
                  aria-label={title}
                  onBrowse={() => {
                    void browseDefaultLocation(kind).then((directory) => {
                      if (directory) update({ [key]: directory });
                    });
                  }}
                  onReset={() => {
                    update({ [key]: null });
                  }}
                  placeholder="Default folder"
                  resetLabel={`Use the default ${title.toLowerCase()}`}
                  value={general[key]}
                />
              </div>
            )}
          </ControlRow>
        ))}
      </SettingsGroup>
      <SettingsGroup title="Capture">
        <ControlRow
          description={
            general.recordScreenwideWindows
              ? "Its windows show in recordings and screenshots."
              : "Its windows are left out, as if they were not there."
          }
          leading={<CaptureArt isIncluded={general.recordScreenwideWindows} />}
          title="Include Screenwide in captures"
        >
          {(controlProps) => (
            <Switch
              {...controlProps}
              isSelected={general.recordScreenwideWindows}
              onChange={(recordScreenwideWindows) => {
                update({ recordScreenwideWindows });
              }}
            />
          )}
        </ControlRow>
        <ControlRow
          description="How long Delayed Screenshot waits."
          title="Screenshot delay"
        >
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label="Screenshot delay"
                items={[3, 5, 10].map((seconds) => ({
                  id: String(seconds),
                  label: `${String(seconds)}s`,
                }))}
                onSelectionChange={(next) => {
                  update({ screenshotDelaySeconds: Number(next) });
                }}
                selected={String(general.screenshotDelaySeconds)}
              />
            </div>
          )}
        </ControlRow>
      </SettingsGroup>
      <SettingsGroup title="Startup">
        <ControlRow title="Start when you sign in">
          {(controlProps) => (
            <Switch
              {...controlProps}
              isSelected={general.launchAtLogin}
              onChange={(launchAtLogin) => {
                update({ launchAtLogin });
              }}
            />
          )}
        </ControlRow>
        <ControlRow
          leading={
            <StartupArt isShown={general.showRecordingControlsOnLaunch} />
          }
          title="Show recording controls on startup"
        >
          {(controlProps) => (
            <Switch
              {...controlProps}
              isSelected={general.showRecordingControlsOnLaunch}
              onChange={(showRecordingControlsOnLaunch) => {
                update({ showRecordingControlsOnLaunch });
              }}
            />
          )}
        </ControlRow>
      </SettingsGroup>
      <SettingsGroup title="About">
        <SoftwareUpdateSetting
          currentVersion="0.1.0"
          error={null}
          onPress={() => {
            setChecked(true);
          }}
          status={checked ? "up-to-date" : "idle"}
          updateVersion={null}
        />
      </SettingsGroup>
    </Pane>
  );
}
