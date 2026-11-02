// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode, type Ref } from "react";
import { useNumberFormatter } from "react-aria";

import { t } from "../../../i18n/i18n";
import {
  type RecordingFps,
  recordingFpsOptions,
} from "../../recording-inputs/types";
import {
  antiFlickerToggle,
  autoZoomToggle,
  confidenceChecksToggle,
  recordingSizeToggle,
} from "../sections/general-rows";
import { SettingsGroup } from "../settings-group";
import { CameraPicture } from "../stage/camera-picture";
import { LiveIsland } from "../stage/live-island";
import { PreviewArea, PreviewStage } from "../stage/preview-stage";
import { RecordingPicture } from "../stage/recording-picture";
import { usePreviewCamera } from "../stage/use-preview-camera";
import { usePreviewSubject } from "../stage/use-preview-subject";

import { ChoiceRow, ToggleRow } from "./setting-rows";
import { SettingsPane } from "./settings-pane";

import type { GeneralSettings } from "../../../bindings/GeneralSettings";

// Recording: how the controls look while a recording is made, and how the
// recording itself is made. The picture is the island itself for the rows
// that change it, so turning the camera and sound off shows the island
// without them; a recording of the sample desktop for the frame rate and
// auto zoom; and your own camera for anti-flicker, where there is one.

type Subject =
  "antiFlicker" | "autoZoom" | "countdown" | "frameRate" | "island";

/** A frame rate the stored settings hold, as one Settings offers. */
const fpsOf = (value: number): RecordingFps =>
  recordingFpsOptions.find((option) => option === value) ?? 60;

export function RecordingPane({
  isSaving,
  onChange,
  paneRef,
  settings,
}: {
  isSaving: boolean;
  onChange: (settings: GeneralSettings) => void;
  settings: GeneralSettings;
  paneRef?: Ref<HTMLElement>;
}) {
  const { follow, subject } = usePreviewSubject<Subject>("island");
  const seconds = useNumberFormatter({
    style: "unit",
    unit: "second",
    unitDisplay: "narrow",
  });
  const update = (changes: Partial<GeneralSettings>) => {
    onChange({ ...settings, ...changes });
  };
  const checks = settings.showRecordingConfidenceChecks;
  const size = settings.showRecordingSize;
  const countdown = settings.recordingCountdownSeconds;
  const fps = fpsOf(settings.recordingFps);
  const antiFlicker = settings.cameraAntiFlicker;
  const autoZoom = settings.autoZoom;
  const camera = usePreviewCamera(fps, antiFlicker);
  // Without a camera to show, anti-flicker has no picture and leaves the
  // one there is.
  const shown = subject === "antiFlicker" && !camera ? "island" : subject;

  const pictures: Record<Subject, ReactNode> = {
    antiFlicker: camera ? (
      <CameraPicture antiFlicker={antiFlicker} camera={camera} fps={fps} />
    ) : null,
    autoZoom: <RecordingPicture scene={{ kind: "autoZoom", on: autoZoom }} />,
    countdown: (
      <LiveIsland
        hasConfidenceChecks={checks}
        scene={{ kind: "countdown", seconds: countdown }}
        showsSize={size}
      />
    ),
    frameRate: <RecordingPicture scene={{ fps, kind: "frameRate" }} />,
    island: (
      <LiveIsland
        hasConfidenceChecks={checks}
        scene={{ kind: "recording" }}
        showsSize={size}
      />
    ),
  };

  return (
    <SettingsPane
      label={t("settings-section-recording")}
      paneRef={paneRef}
      stage={<PreviewStage subject={shown}>{pictures[shown]}</PreviewStage>}
    >
      <SettingsGroup title={t("settings-recording-controls")}>
        <PreviewArea follow={follow("island")}>
          <ToggleRow
            isDisabled={isSaving}
            isSelected={checks}
            item={confidenceChecksToggle()}
            onChange={(showRecordingConfidenceChecks) => {
              update({ showRecordingConfidenceChecks });
            }}
          />
          <ToggleRow
            isDisabled={isSaving}
            isSelected={size}
            item={recordingSizeToggle()}
            onChange={(showRecordingSize) => {
              update({ showRecordingSize });
            }}
          />
        </PreviewArea>
        <PreviewArea follow={follow("countdown")}>
          <ChoiceRow
            isDisabled={isSaving}
            items={[
              { id: "0", label: t("settings-countdown-off") },
              ...[3, 5].map((value) => ({
                id: String(value),
                label: seconds.format(value),
              })),
            ]}
            onChange={(next) => {
              update({ recordingCountdownSeconds: Number(next) });
            }}
            selected={String(countdown)}
            title={t("settings-countdown")}
          />
        </PreviewArea>
      </SettingsGroup>
      <SettingsGroup title={t("settings-recording-video")}>
        <PreviewArea follow={follow("frameRate")}>
          <ChoiceRow
            description={t("settings-frame-rate-description")}
            isDisabled={isSaving}
            items={recordingFpsOptions.map((option) => ({
              id: String(option),
              label: t("settings-fps", { fps: option }),
            }))}
            onChange={(next) => {
              update({ recordingFps: Number(next) });
            }}
            selected={String(fps)}
            title={t("settings-frame-rate")}
          />
        </PreviewArea>
        <PreviewArea follow={follow("autoZoom")}>
          <ToggleRow
            isDisabled={isSaving}
            isSelected={autoZoom}
            item={autoZoomToggle()}
            onChange={(next) => {
              update({ autoZoom: next });
            }}
          />
        </PreviewArea>
        <PreviewArea follow={follow("antiFlicker")}>
          <ToggleRow
            isDisabled={isSaving}
            isSelected={antiFlicker}
            item={antiFlickerToggle()}
            onChange={(cameraAntiFlicker) => {
              update({ cameraAntiFlicker });
            }}
          />
        </PreviewArea>
      </SettingsGroup>
    </SettingsPane>
  );
}
