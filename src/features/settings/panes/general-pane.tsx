// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode, type Ref } from "react";
import { useNumberFormatter } from "react-aria";

import { ControlRow } from "../../../composites/control-row/control-row";
import { PathField } from "../../../composites/path-field/path-field";
import { t } from "../../../i18n/i18n";
import {
  captureToggle,
  controlsOnLaunchToggle,
  folders,
  launchToggle,
} from "../sections/general-rows";
import { useSettingsApi } from "../settings-api-context";
import { SettingsGroup } from "../settings-group";
import { LiveSoftwareUpdateSetting } from "../settings-update-actions";
import { LiveIsland } from "../stage/live-island";
import { PreviewArea, PreviewStage } from "../stage/preview-stage";
import { RecordingPicture } from "../stage/recording-picture";
import { usePreviewSubject } from "../stage/use-preview-subject";

import { ChoiceRow, ToggleRow } from "./setting-rows";
import { SettingsPane } from "./settings-pane";

import type { GeneralSettings } from "../../../bindings/GeneralSettings";

// General: where things are saved, what a capture takes in, and how the app
// starts. The picture shows a recording with Screenwide in it or left out,
// or the desktop just after signing in, where the real island comes out at
// the top of the screen if the controls are shown on startup. The two rows
// of Startup share that one picture with their own values, so moving between
// them carries on with it, or changes only what differs, rather than
// replaying it.

type Subject = "capture" | "controls" | "signIn";

export function GeneralPane({
  isSaving,
  onChange,
  onError,
  paneRef,
  settings,
  updateSetting = <LiveSoftwareUpdateSetting />,
}: {
  isSaving: boolean;
  onChange: (settings: GeneralSettings) => void;
  onError: (message: string) => void;
  settings: GeneralSettings;
  paneRef?: Ref<HTMLElement>;
  updateSetting?: ReactNode;
}) {
  const { browseDefaultLocation } = useSettingsApi();
  const { follow, subject } = usePreviewSubject<Subject>("capture");
  const seconds = useNumberFormatter({
    style: "unit",
    unit: "second",
    unitDisplay: "narrow",
  });
  const update = (changes: Partial<GeneralSettings>) => {
    onChange({ ...settings, ...changes });
  };
  const included = settings.recordScreenwideWindows;
  const launches = settings.launchAtLogin;
  const showsControls = settings.showRecordingControlsOnLaunch;

  // Whether the app starts in the picture: signing in starts it only where
  // set to, and the controls row shows it opening however it was started.
  const appStarts = subject !== "signIn" || launches;
  const picture =
    subject === "capture" ? (
      <RecordingPicture scene={{ included, kind: "includeScreenwide" }} />
    ) : (
      <LiveIsland
        isOn={appStarts}
        scene={{
          kind: "startup",
          showsControls: appStarts && showsControls,
        }}
      />
    );

  return (
    <SettingsPane
      label={t("settings-section-general")}
      paneRef={paneRef}
      stage={
        <PreviewStage subject={subject === "capture" ? "capture" : "startup"}>
          {picture}
        </PreviewStage>
      }
    >
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
      <SettingsGroup title={t("settings-capture")}>
        <PreviewArea follow={follow("capture")}>
          <ToggleRow
            isDisabled={isSaving}
            isSelected={included}
            item={captureToggle()}
            onChange={(recordScreenwideWindows) => {
              update({ recordScreenwideWindows });
            }}
          />
        </PreviewArea>
        <ChoiceRow
          isDisabled={isSaving}
          items={[3, 5, 10].map((value) => ({
            id: String(value),
            label: seconds.format(value),
          }))}
          onChange={(next) => {
            update({ screenshotDelaySeconds: Number(next) });
          }}
          selected={String(settings.screenshotDelaySeconds)}
          title={t("settings-screenshot-delay")}
        />
      </SettingsGroup>
      <SettingsGroup title={t("settings-startup")}>
        <PreviewArea follow={follow("signIn")}>
          <ToggleRow
            isDisabled={isSaving}
            isSelected={launches}
            item={launchToggle()}
            onChange={(launchAtLogin) => {
              update({ launchAtLogin });
            }}
          />
        </PreviewArea>
        <PreviewArea follow={follow("controls")}>
          <ToggleRow
            isDisabled={isSaving}
            isSelected={showsControls}
            item={controlsOnLaunchToggle()}
            onChange={(showRecordingControlsOnLaunch) => {
              update({ showRecordingControlsOnLaunch });
            }}
          />
        </PreviewArea>
      </SettingsGroup>
      {/* Last, as an app's About is: looked at rarely, and not a setting. */}
      <SettingsGroup title={t("settings-about")}>{updateSetting}</SettingsGroup>
    </SettingsPane>
  );
}
