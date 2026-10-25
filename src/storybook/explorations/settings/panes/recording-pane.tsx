// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { ControlRow } from "../../../../composites/control-row/control-row";
import { SettingsGroup } from "../../../../features/settings/settings-group";
import { PillGroup } from "../../../../primitives/pill-group/pill-group";
import { Switch } from "../../../../primitives/switch/switch";
import { LiveIsland } from "../preview/live-island";
import { PreviewRow, PreviewStage, ScreenTop } from "../preview/preview-stage";
import {
  AntiFlickerArt,
  AutoZoomArt,
  FrameRateArt,
} from "../preview/recording-art";
import { usePreviewSubject } from "../preview/use-preview-subject";

import { Pane } from "./pane";

import type { ExploredSettings } from "../settings-state";

// Recording: everything about how a recording looks while it is made and
// once it is made. The stage above the rows is the island itself for the
// rows that change it, so turning confidence checks off shows the island
// without them, and a short loop for the rest.

type Subject =
  "antiFlicker" | "autoZoom" | "countdown" | "frameRate" | "island";

const islandValue = (checks: boolean, size: boolean) =>
  checks && size
    ? "Camera, sound, file size and time"
    : checks
      ? "Camera, sound and time"
      : size
        ? "File size and time"
        : "Only the time";

export function RecordingPane({ settings }: { settings: ExploredSettings }) {
  const { antiFlicker, fps, general, setAntiFlicker, setFps, setGeneral } =
    settings;
  const { follow, subject } = usePreviewSubject<Subject>("island");
  if (!general) return null;
  const checks = general.showRecordingConfidenceChecks;
  const size = general.showRecordingSize;
  const countdown = general.recordingCountdownSeconds;

  const island = (scene: "countdown" | "recording") => (
    <ScreenTop>
      <LiveIsland
        hasConfidenceChecks={checks}
        scene={
          scene === "countdown"
            ? { kind: "countdown", seconds: countdown }
            : { kind: "recording" }
        }
        showsSize={size}
      />
    </ScreenTop>
  );
  const stages: Record<
    Subject,
    { picture: ReactNode; title: string; value: string }
  > = {
    antiFlicker: {
      picture: <AntiFlickerArt isOn={antiFlicker} />,
      title: "Camera under 50 Hz lights",
      value: antiFlicker ? "Steady picture" : "Bands may roll across it",
    },
    autoZoom: {
      picture: <AutoZoomArt isOn={general.autoZoom} />,
      title: "A new recording in the editor",
      value: general.autoZoom
        ? "Zooms in where you click and type"
        : "Shows the whole screen",
    },
    countdown: {
      picture: island("countdown"),
      title: "When you press Record",
      value:
        countdown === 0
          ? "Starts straight away"
          : `Counts down from ${String(countdown)}`,
    },
    frameRate: {
      picture: <FrameRateArt fps={fps} />,
      title: "Frame rate",
      value:
        fps === 60 ? "Smoother, larger files" : "Smaller files, less smooth",
    },
    island: {
      picture: island("recording"),
      title: "While recording",
      value: islandValue(checks, size),
    },
  };
  const stage = stages[subject];

  return (
    <Pane
      label="Recording"
      stage={
        <PreviewStage
          sceneKey={subject}
          title={stage.title}
          value={stage.value}
        >
          {stage.picture}
        </PreviewStage>
      }
    >
      <SettingsGroup title="Recording controls">
        <PreviewRow follow={follow("island")} isActive={subject === "island"}>
          <ControlRow
            description="Check they are working at a glance."
            title="Show camera and sound"
          >
            {(controlProps) => (
              <Switch
                {...controlProps}
                isSelected={checks}
                onChange={(showRecordingConfidenceChecks) => {
                  setGeneral({ ...general, showRecordingConfidenceChecks });
                }}
              />
            )}
          </ControlRow>
        </PreviewRow>
        <PreviewRow follow={follow("island")} isActive={subject === "island"}>
          <ControlRow title="Show file size">
            {(controlProps) => (
              <Switch
                {...controlProps}
                isSelected={size}
                onChange={(showRecordingSize) => {
                  setGeneral({ ...general, showRecordingSize });
                }}
              />
            )}
          </ControlRow>
        </PreviewRow>
        <PreviewRow
          follow={follow("countdown")}
          isActive={subject === "countdown"}
        >
          <ControlRow title="Countdown">
            {(controlProps) => (
              <div {...controlProps} role="group">
                <PillGroup
                  aria-label="Countdown"
                  items={[
                    { id: "0", label: "Off" },
                    { id: "3", label: "3s" },
                    { id: "5", label: "5s" },
                  ]}
                  onSelectionChange={(next) => {
                    setGeneral({
                      ...general,
                      recordingCountdownSeconds: Number(next),
                    });
                  }}
                  selected={String(countdown)}
                />
              </div>
            )}
          </ControlRow>
        </PreviewRow>
      </SettingsGroup>
      <SettingsGroup title="Video">
        <PreviewRow
          follow={follow("frameRate")}
          isActive={subject === "frameRate"}
        >
          <ControlRow title="Frame rate">
            {(controlProps) => (
              <div {...controlProps} role="group">
                <PillGroup
                  aria-label="Frame rate"
                  items={[
                    { id: "30", label: "30 fps" },
                    { id: "60", label: "60 fps" },
                  ]}
                  onSelectionChange={(next) => {
                    setFps(next === "30" ? 30 : 60);
                  }}
                  selected={String(fps)}
                />
              </div>
            )}
          </ControlRow>
        </PreviewRow>
        <PreviewRow
          follow={follow("autoZoom")}
          isActive={subject === "autoZoom"}
        >
          <ControlRow title="Zoom in automatically">
            {(controlProps) => (
              <Switch
                {...controlProps}
                isSelected={general.autoZoom}
                onChange={(autoZoom) => {
                  setGeneral({ ...general, autoZoom });
                }}
              />
            )}
          </ControlRow>
        </PreviewRow>
        <PreviewRow
          follow={follow("antiFlicker")}
          isActive={subject === "antiFlicker"}
        >
          <ControlRow
            description="For cameras that flicker under room lights."
            title="Camera anti-flicker"
          >
            {(controlProps) => (
              <Switch
                {...controlProps}
                isSelected={antiFlicker}
                onChange={setAntiFlicker}
              />
            )}
          </ControlRow>
        </PreviewRow>
      </SettingsGroup>
    </Pane>
  );
}
