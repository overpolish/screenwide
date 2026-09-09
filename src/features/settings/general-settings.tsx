// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PillGroup } from "../../components/base/pill-group/pill-group";
import { Switch } from "../../components/base/switch/switch";
import { PathField } from "../../components/shared/path-field/path-field";
import { Setting } from "../../components/shared/setting/setting";
import { useRecordingInputStore } from "../recording-inputs/store";
import { RecordingFps } from "../recording-inputs/types";

import { useSettingsApi } from "./settings-api-context";

import type { GeneralSettings } from "./types";

const toggles = [
  {
    description:
      "Check your camera, microphone and computer sound are working.",
    key: "showRecordingConfidenceChecks",
    title: "Show camera and sound while recording",
  },
  {
    description: "Show Screenwide's own windows in recordings and screenshots.",
    key: "recordScreenwideWindows",
    title: "Include Screenwide in captures",
  },
  {
    key: "launchAtLogin",
    title: "Start when you sign in",
  },
  {
    key: "showRecordingBarOnLaunch",
    title: "Show recording controls on startup",
  },
] as const;

export function GeneralSettingsPanel({
  isSaving,
  onChange,
  onError,
  settings,
}: {
  isSaving: boolean;
  onChange: (settings: GeneralSettings) => void;
  onError: (message: string) => void;
  settings: GeneralSettings;
}) {
  const { browseDefaultLocation } = useSettingsApi();
  // The frame rate lives with the recording inputs the bar records with, not
  // in the stored preferences; settings only offers the choice.
  const fps = useRecordingInputStore((state) => state.fps);
  const setFps = useRecordingInputStore((state) => state.setFps);
  const update = (changes: Partial<GeneralSettings>) => {
    onChange({ ...settings, ...changes });
  };
  return (
    <div className="gap-layout flex flex-col">
      {(["recording", "screenshot"] as const).map((kind) => {
        const key =
          kind === "recording" ? "recordingDirectory" : "screenshotDirectory";
        const title =
          kind === "recording" ? "Recording folder" : "Screenshot folder";
        return (
          <Setting
            description="You can choose a different folder when saving."
            key={kind}
            title={title}
          >
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
          </Setting>
        );
      })}
      {toggles.map((toggle) => (
        <Setting
          description={"description" in toggle ? toggle.description : undefined}
          key={toggle.key}
          title={toggle.title}
        >
          {(controlProps) => (
            <Switch
              {...controlProps}
              isDisabled={isSaving}
              isSelected={settings[toggle.key]}
              onChange={(selected) => {
                update({ [toggle.key]: selected });
              }}
            />
          )}
        </Setting>
      ))}
      <Setting title="Recording countdown">
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
      </Setting>
      <Setting
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
      </Setting>
    </div>
  );
}
