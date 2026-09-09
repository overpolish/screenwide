// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Meta, type StoryObj } from "@storybook/react-vite";

import { displayThumbnail } from "../../../storybook/display-thumbnail";
import { FeatureStoryStage } from "../../../storybook/feature-story-stage";
import { seedRecordingInputDevices } from "../../../storybook/recording-input-fixtures";
import { MonitorDetails, WindowDetails } from "../../recording-sources/types";

import { RecordingBar } from "./recording-bar";

const builtInDisplay: MonitorDetails = {
  id: 1,
  isBuiltin: true,
  isPrimary: true,
  layoutPosition: { x: 0, y: 0 },
  layoutSize: { height: 982, width: 1512 },
  name: "Built-in Display",
  physicalPosition: { x: 0, y: 0 },
  physicalSize: { height: 1964, width: 3024 },
  position: { x: 0, y: 0 },
  scaleFactor: 2,
  size: { height: 982, width: 1512 },
};

/** Stands in for the captured still of the display that is chosen. */
const displayThumbnails: Record<number, string> = {
  [builtInDisplay.id]: displayThumbnail("#3b6fd4", "#8f5bd6"),
};

/** A window whose app has no icon on hand, so the segment keeps its glyph. */
const longTitledWindow: WindowDetails = {
  appIconPath: null,
  appName: "Safari",
  id: 42,
  pid: 900,
  position: { x: 120, y: 80 },
  size: { height: 720, width: 1180 },
  thumbnailPath: null,
  title: "Quarterly Planning Notes and Follow-ups — Safari",
};

const meta = {
  args: {
    hasSelectedMonitor: true,
    monitorThumbnails: displayThumbnails,
    onScrollingScreenshot: () => undefined,
    selectedMonitor: builtInDisplay,
    selectedWindow: null,
  },
  component: RecordingBar,
  decorators: [
    (Story, context) => {
      // Storybook's Tauri stub lists no devices, so the inputs would all
      // read as absent; the store is seeded with a typical Mac's instead.
      seedRecordingInputDevices();
      return (
        <FeatureStoryStage height={56} viewMode={context.viewMode} width={843}>
          <Story />
        </FeatureStoryStage>
      );
    },
  ],
  parameters: {
    layout: "fullscreen",
  },
  title: "Features/Recording Bar",
} satisfies Meta<typeof RecordingBar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const NoMonitorSelected: Story = {
  args: {
    hasSelectedMonitor: false,
    monitorThumbnails: {},
    selectedMonitor: null,
  },
};

/** Screen recording denied: setting up is disabled, and the record button
 * carries a lock that opens the permissions window. */
export const PermissionsLocked: Story = {
  args: { isLocked: true, isScreenshotLocked: true },
};

/** Accessibility denied but screen recording granted: stills still work. */
export const ScreenshotOnly: Story = {
  args: { isLocked: true },
};

/** Mid-capture: the button is inert and pulsing until the file exists. The
 * three screenshot actions share these states, so one set stands for all. */
export const ScreenshotPending: Story = {
  args: { screenshotState: "pending" },
};

export const ScreenshotDone: Story = {
  args: { screenshotState: "done" },
};

export const ScreenshotFailed: Story = {
  args: { screenshotState: "failed" },
};

/**
 * An open screenshot workspace has a window of its own and blocks nothing:
 * both capture buttons stay live.
 */
export const ScreenshotWorkspaceOpen: Story = {
  args: { pendingEditors: { recording: false, screenshot: true } },
};

/**
 * A recording waiting for export: the record button stays pressable and shows
 * that window instead, while screenshots go on being taken beside it.
 */
export const RecordingWorkspaceOpen: Story = {
  args: { pendingEditors: { recording: true, screenshot: false } },
};

export const OptionalPermissionsLocked: Story = {
  args: { isCameraLocked: true, isMicrophoneLocked: true },
};

export const InputsEnabled: Story = {
  args: {
    initialInputs: {
      camera: true,
      microphone: true,
      systemAudio: true,
    },
  },
};

export const MissingEnabledInputs: Story = {
  args: {
    hasCameraWarning: true,
    hasMicrophoneWarning: true,
    hasSystemAudioWarning: true,
    initialInputs: {
      camera: true,
      microphone: true,
      systemAudio: true,
    },
  },
};

/** Missing sources stay quiet until their corresponding input is enabled. */
export const MissingDisabledInputs: Story = {
  args: {
    hasCameraWarning: true,
    hasMicrophoneWarning: true,
    hasSystemAudioWarning: true,
  },
};

export const Region: Story = {
  args: { initialMode: "region" },
};

export const Window: Story = {
  args: { initialMode: "window" },
};

export const WindowSelected: Story = {
  args: {
    hasSelectedWindow: true,
    initialMode: "window",
    selectedWindow: longTitledWindow,
  },
};

export const CameraOnly: Story = {
  args: { initialMode: "camera" },
};

export const CameraOnlyPreservesScreenCameraOff: Story = {
  args: { initialInputs: { camera: false }, initialMode: "camera" },
};

export const CameraOnlyMissing: Story = {
  args: {
    hasCameraWarning: true,
    initialMode: "camera",
  },
};

export const CameraOnlyPermissionLocked: Story = {
  args: { initialMode: "camera", isCameraLocked: true },
};

export const AudioOnlyDisabled: Story = {
  args: { initialMode: "audio" },
};

export const AudioOnlyWithMicrophone: Story = {
  args: {
    initialInputs: { microphone: true },
    initialMode: "audio",
  },
};

export const AudioOnlyWithSystemAudio: Story = {
  args: {
    initialInputs: { systemAudio: true },
    initialMode: "audio",
  },
};

export const AudioOnlyWithAllSourcesMissing: Story = {
  args: {
    hasMicrophoneWarning: true,
    hasSystemAudioWarning: true,
    initialInputs: { microphone: true, systemAudio: true },
    initialMode: "audio",
  },
};

export const AudioOnlyWithOneValidSource: Story = {
  args: {
    hasMicrophoneWarning: true,
    initialInputs: { microphone: true, systemAudio: true },
    initialMode: "audio",
  },
};

export const Starting: Story = {
  args: { status: "starting" },
};
