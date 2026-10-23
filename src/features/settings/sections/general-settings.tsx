// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";
import { useNumberFormatter } from "react-aria";

import { ControlRow } from "../../../composites/control-row/control-row";
import { PathField } from "../../../composites/path-field/path-field";
import { t } from "../../../i18n/i18n";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
import { Switch } from "../../../primitives/switch/switch";
import { useRecordingInputStore } from "../../recording-inputs/store";
import { RecordingFps } from "../../recording-inputs/types";
import { useSettingsApi } from "../settings-api-context";
import { SettingsGroup } from "../settings-group";
import { LiveSoftwareUpdateSetting } from "../settings-update-actions";

import {
  autoZoomToggle,
  captureToggle,
  folders,
  recordingSizeToggle,
  recordingToggle,
  startupToggles,
  type ToggleItem,
} from "./general-rows";

import type { GeneralSettings } from "../../../bindings/GeneralSettings";

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
  // The frame rate and anti-flicker live with the recording inputs the
  // controls record with, not in the stored preferences; settings only
  // offers the choice.
  const fps = useRecordingInputStore((state) => state.fps);
  const setFps = useRecordingInputStore((state) => state.setFps);
  const cameraPal = useRecordingInputStore((state) => state.cameraPal);
  const setCameraPal = useRecordingInputStore((state) => state.setCameraPal);
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
    <div className="flex flex-col gap-group">
      <SettingsGroup title={t("settings-saving")}>
        {folders().map(({ description, key, kind, reset, title }) => (
          <ControlRow description={description} key={kind} title={title}>
            {(controlProps) => (
              <div {...controlProps} role="group">
                <PathField
                  aria-label={title}
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
                  onReset={() => {
                    update({ [key]: null });
                  }}
                  placeholder={t("settings-default-folder")}
                  resetLabel={reset}
                  value={settings[key]}
                />
              </div>
            )}
          </ControlRow>
        ))}
      </SettingsGroup>
      <SettingsGroup title={t("settings-recording")}>
        {toggle(recordingToggle())}
        {toggle(recordingSizeToggle())}
        {toggle(autoZoomToggle())}
        <ControlRow title={t("settings-countdown")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label={t("settings-countdown")}
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
        <ControlRow
          description={t("settings-anti-flicker-description")}
          title={t("settings-anti-flicker")}
        >
          {(controlProps) => (
            <Switch
              {...controlProps}
              isSelected={cameraPal}
              onChange={setCameraPal}
            />
          )}
        </ControlRow>
      </SettingsGroup>
      <SettingsGroup title={t("settings-capture")}>
        {toggle(captureToggle())}
        <ControlRow title={t("settings-screenshot-delay")}>
          {(controlProps) => (
            <div {...controlProps} role="group">
              <PillGroup
                aria-label={t("settings-screenshot-delay")}
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
      </SettingsGroup>
      <SettingsGroup title={t("settings-startup")}>
        {startupToggles().map((item) => toggle(item))}
      </SettingsGroup>
      {/* Last, as an app's About is: looked at rarely, and not a setting. */}
      <SettingsGroup title={t("settings-about")}>{updateSetting}</SettingsGroup>
    </div>
  );
}
