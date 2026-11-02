// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../../storybook/feature-story-stage";
import { EditorArtifact } from "../types";

import { EditorPanel } from "./editor-panel";
import {
  AudioRecordingStoryPanel,
  RecordingStoryPanel,
  ScreenshotStoryPanel,
} from "./editor-panel-story-panels";
import {
  recording,
  screenshot,
  screenshotOutput,
} from "./editor-story-fixtures";

import type { Decorator, Meta, StoryObj } from "@storybook/react-vite";

const screenPreviewLayout = {
  height: 720,
  panes: [
    {
      height: 720,
      kind: "screen" as const,
      sourceHeight: 2160,
      sourceWidth: 3840,
      width: 1280,
      x: 0,
      y: 0,
    },
  ],
  width: 1280,
};

const cameraPreviewLayout = {
  height: 720,
  panes: [
    ...screenPreviewLayout.panes,
    {
      height: 720,
      kind: "camera" as const,
      sourceHeight: 1080,
      sourceWidth: 1920,
      width: 1280,
      x: 1280,
      y: 0,
    },
  ],
  width: 2560,
};

const cameraOnlyPreviewLayout = {
  height: 720,
  panes: [
    {
      height: 720,
      kind: "camera" as const,
      sourceHeight: 1080,
      sourceWidth: 1920,
      width: 1280,
      x: 0,
      y: 0,
    },
  ],
  width: 1280,
};

const audioPreviewLayout = {
  height: 0,
  panes: [],
  width: 0,
};

// The native compositor paints every preview, and cannot run here, so a rule
// in `.storybook/styles.css` labels the empty viewport inside
// `.sb-gpu-preview`. `Opening` has no viewport and is left unwrapped.
const withGpuPreviewPlaceholder: Decorator = (Story, context) => {
  const artifact = context.args.artifact as EditorArtifact | null | undefined;
  if (!artifact) return <Story />;
  return (
    <div className="sb-gpu-preview" style={{ display: "contents" }}>
      <Story />
    </div>
  );
};

const events = {
  onBakeCameraChange: fn(),
  onBrowse: fn(),
  onCameraCompressionChange: fn(),
  onCameraKeyChange: fn(),
  onCameraOverlayChange: fn(),
  onCameraResolutionScaleChange: fn(),
  onCancel: fn(),
  onCancelSave: fn(),
  onCanvasResize: fn(),
  onCollapseAudioChange: fn(),
  onCompressionChange: fn(),
  onCopy: fn(),
  onCursorEffectsChange: fn(),
  onDeleteProject: fn(),
  onDeleteProjectAfterExportChange: fn(),
  onEnabledTracksChange: fn(),
  onEnabledVideoTracksChange: fn(),
  onFileStemChange: fn(),
  onKeyboardEffectsChange: fn(),
  onMinimize: fn(),
  onRecordingOutputChange: fn(),
  onRecordingTimelineEditChange: fn(),
  onRenameProject: fn(),
  onResolutionScaleChange: fn(),
  onSave: fn(),
  onScreenshotFormatChange: fn(),
  onScreenshotOutputChange: fn(),
  onSelectedScreenshotItemChange: fn(),
  onSelectedTrackChange: fn(),
  onSubtitlesChange: fn(),
  onToggleMaximize: fn(),
  onTrackVolumeChange: fn(),
};

const meta = {
  args: {
    ...events,
    artifact: screenshot,
    directory: "/Users/dom/Desktop",
    fileStem: screenshot.suggestedFileStem,
    screenshotOutput: { ...screenshotOutput, items: [] },
  },
  component: EditorPanel,
  decorators: [
    withGpuPreviewPlaceholder,
    featureStage(({ args }) => ({
      window:
        args.artifact?.kind === "screenshot"
          ? "editor-screenshot"
          : "editor-recording",
    })),
  ],
  parameters: {
    controls: { exclude: Object.keys(events) },
    layout: "fullscreen",
  },
  title: "Features/Editor/Panel/Editor",
} satisfies Meta<typeof EditorPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Opening: Story = {
  args: { artifact: null, fileStem: "" },
};

export const Screenshot: Story = {
  render: (args) => <ScreenshotStoryPanel {...args} />,
};

export const Recording: Story = {
  args: {
    artifact: recording,
    compression: 2,
    enabledAudioTrackCount: 2,
    enabledVideoTracks: ["primary"],
    fileStem: recording.suggestedFileStem,
    recordingPreviewLayout: screenPreviewLayout,
    recordingPreviewTracks: [
      {
        kind: "system-audio",
        label: "System audio",
        streamIndex: 0,
        waveform: Array.from(
          { length: 96 },
          (_, index) => Math.abs(Math.sin(index * 0.31)) * 0.8,
        ),
      },
      {
        kind: "microphone",
        label: "Microphone",
        streamIndex: 1,
        waveform: Array.from(
          { length: 96 },
          (_, index) => Math.abs(Math.sin(index * 0.17)) * 0.55,
        ),
      },
    ],
    resolutionScalePercent: 150,
  },
  render: (args) => <RecordingStoryPanel {...args} />,
};

export const Saving: Story = {
  args: {
    ...Recording.args,
    isSaving: true,
  },
  render: (args) => <RecordingStoryPanel {...args} />,
};

export const LongProjectName: Story = {
  args: {
    ...Recording.args,
    artifact: {
      ...recording,
      suggestedFileStem:
        "Screenwide recording with a deliberately long editable project name 2026-08-08",
    },
  },
  render: (args) => <RecordingStoryPanel {...args} />,
};

export const RecordingWithNothingSelected: Story = {
  args: {
    ...Recording.args,
    enabledAudioTrackCount: 0,
    enabledStreamIndices: [],
    enabledVideoTracks: [],
  },
};

export const CameraRecording: Story = {
  args: {
    ...Recording.args,
    artifact: {
      ...recording,
      cursorDataVersion: null,
      hasCursorData: false,
      hasKeyboardData: false,
      height: 1080,
      keyboardDataVersion: null,
      primaryKind: "camera",
      sourceScalePercent: 100,
      width: 1920,
    },
    recordingPreviewLayout: cameraOnlyPreviewLayout,
    resolutionScalePercent: 75,
  },
  render: (args) => <RecordingStoryPanel {...args} />,
};

export const AudioRecording: Story = {
  args: {
    ...Recording.args,
    artifact: {
      ...recording,
      canCompress: false,
      cursorDataVersion: null,
      extension: "m4a",
      hasCursorData: false,
      hasKeyboardData: false,
      height: 0,
      keyboardDataVersion: null,
      path: "/tmp/Recordings/audio-20260808-143205.000.mov",
      primaryKind: "audio",
      sourceScalePercent: 100,
      width: 0,
    },
    enabledVideoTracks: [],
    recordingPreviewLayout: audioPreviewLayout,
    resolutionScalePercent: 100,
  },
  render: (args) => <AudioRecordingStoryPanel {...args} />,
};

export const RecordingWithCamera: Story = {
  args: {
    ...Recording.args,
    artifact: {
      ...recording,
      camera: {
        durationMs: recording.durationMs,
        height: 1080,
        originalSizeBytes: 18_400_000,
        path: "/tmp/Recordings/camera-20260808-143205.000.mov",
        width: 1920,
      },
    },
    cameraCompression: 2,
    cameraResolutionScalePercent: 100,
    enabledVideoTracks: ["primary", "camera"],
    recordingPreviewLayout: cameraPreviewLayout,
  },
  render: (args) => <RecordingStoryPanel {...args} />,
};

export const RecoveredRecording: Story = {
  args: {
    artifact: {
      ...recording,
      audioTracks: [],
      durationMs: 0,
      height: 0,
      originalSizeBytes: 0,
      width: 0,
    },
    enabledVideoTracks: ["primary"],
    fileStem: recording.suggestedFileStem,
    recordingPreviewLayout: screenPreviewLayout,
  },
};
