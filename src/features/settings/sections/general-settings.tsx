// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";
import { useNumberFormatter } from "react-aria";

import { GroupBox } from "../../../components/base/group-box/group-box";
import { PillGroup } from "../../../components/base/pill-group/pill-group";
import { Switch } from "../../../components/base/switch/switch";
import { ControlRow } from "../../../components/shared/control-row/control-row";
import { PathField } from "../../../components/shared/path-field/path-field";
import { t } from "../../../i18n/i18n";
import { useRecordingInputStore } from "../../recording-inputs/store";
import { RecordingFps } from "../../recording-inputs/types";
import { useSettingsApi } from "../settings-api-context";
import { LiveSoftwareUpdateSetting } from "../settings-update-actions";

import type { GeneralSettings } from "../../../bindings/GeneralSettings";

/** The preferences a Switch row can drive. */
type ToggleKey = {
  [K in keyof GeneralSettings]: GeneralSettings[K] extends boolean ? K : never;
}[keyof GeneralSettings];

type ToggleItem = { key: ToggleKey; title: string; description?: string };

const recordingToggle = (): ToggleItem => ({
  description: t("settings-confidence-checks-description"),
  key: "showRecordingConfidenceChecks",
  title: t("settings-confidence-checks"),
});

const autoZoomToggle = (): ToggleItem => ({
  description: t("settings-auto-zoom-description"),
  key: "autoZoom",
  title: t("settings-auto-zoom"),
});

const captureToggle = (): ToggleItem => ({
  description: t("settings-include-app-description"),
  key: "recordScreenwideWindows",
  title: t("settings-include-app"),
});

const startupToggles = (): ToggleItem[] => [
  { key: "launchAtLogin", title: t("settings-launch-at-login") },
  {
    key: "showRecordingBarOnLaunch",
    title: t("settings-show-bar-on-launch"),
  },
];

/** The folders Settings names: where projects live, and where exports go. */
const folders = () =>
  [
    {
      description: t("settings-projects-folder-description"),
      key: "projectDirectory",
      kind: "project",
      reset: t("settings-projects-folder-reset"),
      title: t("settings-projects-folder"),
    },
    {
      description: t("settings-export-folder-description"),
      key: "recordingDirectory",
      kind: "recording",
      reset: t("settings-recording-folder-reset"),
      title: t("settings-recording-folder"),
    },
    {
      description: t("settings-export-folder-description"),
      key: "screenshotDirectory",
      kind: "screenshot",
      reset: t("settings-screenshot-folder-reset"),
      title: t("settings-screenshot-folder"),
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
  const seconds = useNumberFormatter({
    style: "unit",
    unit: "second",
    unitDisplay: "narrow",
  });
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
  const secondsItem = (value: number) => ({
    id: String(value),
    label: seconds.format(value),
  });
  return (
    <div className="gap-layout flex flex-col">
      <GroupBox title={t("settings-software-update")}>{updateSetting}</GroupBox>
      <GroupBox title={t("settings-appearance")}>
        <ControlRow
          description={t("settings-accent-description")}
          title={t("settings-accent")}
        >
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label={t("settings-accent")}
                display="label"
                isDisabled={isSaving}
                items={[
                  { id: "system", label: t("settings-accent-system") },
                  { id: "screenwide", label: t("settings-accent-app") },
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
      <GroupBox title={t("settings-saving")}>
        {folders().map(({ description, key, kind, reset, title }) => (
          <ControlRow description={description} key={kind} title={title}>
            {(controlProps) => (
              <div {...controlProps} role="group">
                <PathField
                  aria-label={title}
                  emptyLabel={t("settings-default-folder")}
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
                    label: reset,
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
      <GroupBox title={t("settings-recording")}>
        {toggle(recordingToggle())}
        {toggle(autoZoomToggle())}
        <ControlRow title={t("settings-countdown")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label={t("settings-countdown")}
                display="label"
                isDisabled={isSaving}
                items={[
                  { id: "0", label: t("settings-countdown-off") },
                  secondsItem(3),
                  secondsItem(5),
                ]}
                onSelectionChange={(next) => {
                  update({
                    recordingCountdownSeconds: Number(next),
                  });
                }}
                selected={String(settings.recordingCountdownSeconds)}
              />
            </div>
          )}
        </ControlRow>
        <ControlRow
          description={t("settings-frame-rate-description")}
          title={t("settings-frame-rate")}
        >
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label={t("settings-frame-rate")}
                display="label"
                items={[
                  { id: "30", label: t("settings-fps", { fps: 30 }) },
                  { id: "60", label: t("settings-fps", { fps: 60 }) },
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
      <GroupBox title={t("settings-capture")}>
        {toggle(captureToggle())}
        <ControlRow title={t("settings-screenshot-delay")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label={t("settings-screenshot-delay")}
                display="label"
                isDisabled={isSaving}
                items={[secondsItem(3), secondsItem(5), secondsItem(10)]}
                onSelectionChange={(next) => {
                  update({
                    screenshotDelaySeconds: Number(next),
                  });
                }}
                selected={String(settings.screenshotDelaySeconds)}
              />
            </div>
          )}
        </ControlRow>
      </GroupBox>
      <GroupBox title={t("settings-startup")}>
        {startupToggles().map((item) => toggle(item))}
      </GroupBox>
    </div>
  );
}
