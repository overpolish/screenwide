// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { FeatureStoryStage } from "../../../storybook/feature-story-stage";
import { recording, screenshot } from "../components/editor-story-fixtures";
import {
  defaultRecordingOutput,
  RecordingOutputSettings,
} from "../screenshot-output";
import { EditorArtifact } from "../types";

import { ExportOptionsWindowView } from "./export-options-window-view";
import { ExportProgressProps } from "./export-progress";

import type { Meta, StoryObj } from "@storybook/react-vite";

const cameraRecording: EditorArtifact = {
  ...recording,
  camera: {
    durationMs: recording.durationMs,
    height: 1080,
    originalSizeBytes: 18_400_000,
    path: "/tmp/Recordings/camera-20260808-143205.000.mov",
    width: 1920,
  },
};

const audioRecording: EditorArtifact = {
  ...recording,
  canCompress: false,
  extension: "m4a",
  height: 0,
  primaryKind: "audio",
  sourceScalePercent: 100,
  width: 0,
};

/** Story-only wiring: the settings are local state and nothing is saved. */
function EditorExportPreview({
  artifact,
  enabledAudioTrackCount = 2,
  estimatedSizeBytes = 74_200_000,
  extension,
  includeCamera = false,
  progress,
  recordingOutput,
}: {
  artifact: EditorArtifact;
  enabledAudioTrackCount?: number;
  estimatedSizeBytes?: number | null;
  extension?: string;
  includeCamera?: boolean;
  /** When given, the window shows the running save instead of the form. */
  progress?: ExportProgressProps;
  recordingOutput?: RecordingOutputSettings;
}) {
  const [fileStem, setFileStem] = useState(artifact.suggestedFileStem);
  const [directory, setDirectory] = useState<string | null>(
    "/Users/dom/Desktop",
  );
  const [cameraCompression, setCameraCompression] = useState(2);
  const [cameraResolution, setCameraResolution] = useState(100);
  const [collapseAudio, setCollapseAudio] = useState(false);
  const [compression, setCompression] = useState(2);
  const [openLocation, setOpenLocation] = useState(true);
  const [resolution, setResolution] = useState(200);

  return (
    <ExportOptionsWindowView
      form={{
        artifact,
        cameraCompression,
        cameraResolutionScalePercent: cameraResolution,
        collapseAudio,
        compression,
        directory,
        enabledAudioTrackCount,
        estimatedSizeBytes,
        extension: extension ?? artifact.extension,
        fileStem,
        includeCamera,
        onBrowse: () => {
          setDirectory((current) =>
            current?.endsWith("Desktop") ? "/Users/dom/Movies" : null,
          );
        },
        onCameraCompressionChange: setCameraCompression,
        onCameraResolutionScaleChange: setCameraResolution,
        onCancel: () => undefined,
        onCollapseAudioChange: setCollapseAudio,
        onCompressionChange: setCompression,
        onExport: () => undefined,
        onFileStemChange: setFileStem,
        onOpenLocationAfterExportChange: setOpenLocation,
        onResolutionScaleChange: setResolution,
        openLocationAfterExport: openLocation,
        recordingOutput,
        resolutionScalePercent: resolution,
      }}
      isSaving={Boolean(progress)}
      onClose={() => undefined}
      progress={progress}
    />
  );
}

const meta = {
  args: { artifact: recording },
  component: EditorExportPreview,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage
        height={context.args.artifact.kind === "recording" ? 700 : 480}
        viewMode={context.viewMode}
        width={480}
      >
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Editor/Export",
} satisfies Meta<typeof EditorExportPreview>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Video: Story = {};

export const Screenshot: Story = {
  args: { artifact: screenshot, estimatedSizeBytes: null },
};

/** The camera stays a separate output, so it gets its own size and quality. */
export const SeparateCamera: Story = {
  args: { artifact: cameraRecording, includeCamera: true },
};

export const EditedFrames: Story = {
  args: {
    artifact: cameraRecording,
    includeCamera: true,
    recordingOutput: defaultRecordingOutput({
      camera: { height: 1280, width: 720 },
      primary: { height: 1200, width: 1600 },
    }),
  },
};

export const AudioOnly: Story = {
  args: {
    artifact: audioRecording,
    estimatedSizeBytes: null,
    extension: "m4a",
  },
};

const savingRecording: ExportProgressProps = {
  cancellable: true,
  etaSeconds: 12,
  isAudioOnly: false,
  isCancelingSave: false,
  isRecording: true,
  phase: "camera",
  progress: 42,
};

export const RecordingProgress: Story = {
  args: { progress: savingRecording },
};

/** The last mux reports nothing, so the ring carries the wait on its own. */
export const RecordingFinalizing: Story = {
  args: {
    progress: {
      ...savingRecording,
      etaSeconds: null,
      phase: "finalizing",
      progress: null,
    },
  },
};

/** A screenshot is written in one pass, so there is nothing to cancel. */
export const ScreenshotProgress: Story = {
  args: {
    artifact: screenshot,
    progress: {
      ...savingRecording,
      cancellable: false,
      etaSeconds: null,
      isRecording: false,
      phase: "recording",
      progress: null,
    },
  },
};
