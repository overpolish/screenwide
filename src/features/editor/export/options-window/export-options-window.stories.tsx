// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { recording, screenshot } from "../../panel/editor-story-fixtures";
import {
  defaultRecordingOutput,
  RecordingOutputSettings,
} from "../../screenshot/screenshot-output";
import { EditorArtifact } from "../../types";
import { CameraOutput } from "../camera-output";
import { ExportPreviewCompare } from "../preview/export-preview-compare";
import { storyFrames } from "../preview/export-preview-story-fixtures";

import { ExportOptionsWindowView } from "./export-options-window-view";
import { ExportProgressProps } from "./export-progress";

import type { ExportSubtitles } from "../../../../bindings/ExportSubtitles";
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
  cameraOutput = null,
  enabledAudioTrackCount = 2,
  estimatedSizeBytes = 74_200_000,
  extension,
  hasTranscript = true,
  includeCamera = false,
  isEstimatingSize = false,
  progress,
  recordingOutput,
  withPreview = false,
}: {
  artifact: EditorArtifact;
  cameraOutput?: CameraOutput | null;
  enabledAudioTrackCount?: number;
  estimatedSizeBytes?: number | null;
  extension?: string;
  hasTranscript?: boolean;
  includeCamera?: boolean;
  isEstimatingSize?: boolean;
  /** When given, the window shows the running save instead of the form. */
  progress?: ExportProgressProps;
  recordingOutput?: RecordingOutputSettings;
  /** The frame at Original and as exported, beside the form. */
  withPreview?: boolean;
}) {
  const [frames] = useState(() => (withPreview ? storyFrames() : null));
  const [fileStem, setFileStem] = useState(artifact.suggestedFileStem);
  const [directory, setDirectory] = useState<string | null>(
    "/Users/dom/Desktop",
  );
  const [cameraCompression, setCameraCompression] = useState(2);
  const [cameraResolution, setCameraResolution] = useState(100);
  const [collapseAudio, setCollapseAudio] = useState(false);
  const [compression, setCompression] = useState(2);
  const [deleteProject, setDeleteProject] = useState(false);
  const [openLocation, setOpenLocation] = useState(true);
  const [resolution, setResolution] = useState(200);
  const [subtitles, setSubtitles] = useState<ExportSubtitles>("srt");

  return (
    <ExportOptionsWindowView
      form={{
        artifact,
        cameraCompression,
        cameraOutput,
        cameraResolutionScalePercent: cameraResolution,
        collapseAudio,
        compression,
        deleteProjectAfterExport: deleteProject,
        directory,
        enabledAudioTrackCount,
        estimatedSizeBytes,
        extension: extension ?? artifact.extension,
        fileStem,
        hasTranscript,
        includeCamera,
        isEstimatingSize,
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
        onDeleteProjectAfterExportChange: setDeleteProject,
        onExport: () => undefined,
        onFileStemChange: setFileStem,
        onOpenLocationAfterExportChange: setOpenLocation,
        onResolutionScaleChange: setResolution,
        onSubtitlesChange: setSubtitles,
        openLocationAfterExport: openLocation,
        recordingOutput,
        resolutionScalePercent: resolution,
        subtitles,
      }}
      isSaving={Boolean(progress)}
      onClose={() => undefined}
      preview={
        frames ? (
          <ExportPreviewCompare
            className="flex-1"
            exported={frames.exported}
            failed={false}
            isUpdating={false}
            original={frames.original}
          />
        ) : undefined
      }
      progress={progress}
    />
  );
}

const meta = {
  args: { artifact: recording },
  component: EditorExportPreview,
  decorators: [
    // The window fits its height to the form at runtime, so the config's
    // size does not apply.
    featureStage(({ args }) => ({
      height: args.artifact.kind === "recording" ? 700 : 480,
      width: args.withPreview ? 920 : 480,
    })),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Editor/Export",
} satisfies Meta<typeof EditorExportPreview>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Video: Story = {};

/** The frame under the playhead at Original and as it will be exported,
 * beside the form. */
export const Preview: Story = {
  args: { withPreview: true },
};

/** No transcript yet, so there is nothing to make subtitles from. */
export const Untranscribed: Story = {
  args: { hasTranscript: false },
};

export const EstimatingSize: Story = {
  args: { isEstimatingSize: true },
};

export const Screenshot: Story = {
  args: { artifact: screenshot, estimatedSizeBytes: null },
};

/** The camera saved as a file of its own, so it gets its own size and
 * quality. */
export const SeparateCamera: Story = {
  args: {
    artifact: cameraRecording,
    cameraOutput: "separate",
    includeCamera: true,
  },
};

/** The camera drawn into the screen's video: one file, so one size and one
 * quality. */
export const CombinedCamera: Story = {
  args: {
    artifact: cameraRecording,
    cameraOutput: "combined",
    includeCamera: true,
  },
};

export const EditedFrames: Story = {
  args: {
    artifact: cameraRecording,
    cameraOutput: "separate",
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
