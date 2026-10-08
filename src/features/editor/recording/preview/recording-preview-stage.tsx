// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject } from "react";

import { CircularProgress } from "../../../../components/base/circular-progress/circular-progress";
import { Text } from "../../../../components/base/text/text";
import { t } from "../../../../i18n/i18n";
import {
  RecordingOutputSettings,
  defaultScreenshotOutput,
} from "../../screenshot/screenshot-output";
import { NativeAudioRibbon } from "../../timeline/audio/native-audio-ribbon";
import { useRecordingPreviewTracks } from "../../timeline/tracks/use-recording-preview-tracks";

import { NativeRecordingWorkspaceViewport } from "./native-recording-workspace-viewport";
import { RecordingCanvasTool } from "./recording-crop-toggle";
import {
  bakedCameraWorkspace,
  recordingLayoutWorkspace,
  recordingOutputWorkspace,
} from "./recording-workspaces";
import { useRecordingPreviewTransport } from "./use-recording-preview-transport";

import type { ResolvedScrubPreviewProps } from "./recording-preview-props";

/** One line of trouble under the preview: the audio, the player and the frame
 * copier each report their own, and each reads the same way. */
function PreviewError({ message }: { message?: string | null }) {
  if (!message) return null;
  return (
    <Text
      className="px-window-inset pb-control-inset text-error"
      variant="footnote"
    >
      {message}
    </Text>
  );
}

/** The picture above the timeline: the workspace arranged the way the tracks
 * on screen ask, with the audio ribbon standing in when there is no video at
 * all. */
export function RecordingPreviewStage({
  activeRecordingOutput,
  artifactId,
  audioError,
  audioTracks,
  audioVolumeByStream,
  cameraPane,
  canPreviewBakedCamera,
  canvasTool,
  copyError,
  enabledTracks,
  isPreparingPreview,
  layout,
  player,
  previewLayout,
  screenCanvasRef,
  screenPane,
  visibleCanvasRefs,
  visibleLayout,
  visiblePaneEntries,
}: Pick<
  ResolvedScrubPreviewProps,
  | "artifactId"
  | "audioError"
  | "audioTracks"
  | "isPreparingPreview"
  | "previewLayout"
> &
  Pick<
    ReturnType<typeof useRecordingPreviewTransport>,
    | "cameraPane"
    | "layout"
    | "player"
    | "screenPane"
    | "visibleCanvasRefs"
    | "visibleLayout"
    | "visiblePaneEntries"
  > &
  Pick<
    ReturnType<typeof useRecordingPreviewTracks>,
    "audioVolumeByStream" | "enabledTracks"
  > & {
    canPreviewBakedCamera: boolean;
    canvasTool: RecordingCanvasTool;
    screenCanvasRef: RefObject<HTMLCanvasElement | null>;
    activeRecordingOutput?: RecordingOutputSettings;
    copyError?: string | null;
  }) {
  const isPreparing =
    previewLayout === undefined && (player.isPreparing || isPreparingPreview);
  const isSelecting = canvasTool === "select";
  const shownLayout =
    visibleLayout && visibleLayout.panes.length > 0 ? visibleLayout : null;
  // Every arrangement shares one viewport element. Swapping One video for
  // Separate files would otherwise mount a fresh viewport at zero size, and
  // the native layout reads unsized markers as a recording with no video:
  // it hides every pane until the new viewport has measured itself.
  const workspace = !layout
    ? null
    : canPreviewBakedCamera && screenPane && cameraPane
      ? {
          ...bakedCameraWorkspace(
            // Draft output keeps the frame and workspace on the resize
            // before it reaches the editor window's state.
            activeRecordingOutput?.primary ??
              defaultScreenshotOutput(screenPane.width, screenPane.height),
            screenCanvasRef,
          ),
          isBusy: isPreparing,
          isSelecting,
        }
      : shownLayout && activeRecordingOutput
        ? {
            ...recordingOutputWorkspace(
              visiblePaneEntries,
              activeRecordingOutput,
            ),
            isBusy: false,
            isSelecting,
          }
        : shownLayout
          ? {
              ...recordingLayoutWorkspace(shownLayout, visibleCanvasRefs),
              isBusy: isPreparing,
              isSelecting: false,
            }
          : null;
  return (
    <section className="relative flex min-h-0 min-w-0 grow flex-col">
      <div className="flex min-h-0 grow items-stretch justify-center">
        {!layout ? (
          <div className="flex grow items-center justify-center gap-3 text-xs text-muted">
            <CircularProgress
              aria-label={t("editor-recording-preparing-recording")}
              isIndeterminate
            />
            {t("editor-recording-preparing-recording")}
          </div>
        ) : workspace ? (
          <div className="flex min-h-0 min-w-0 grow flex-col">
            <NativeRecordingWorkspaceViewport {...workspace} />
          </div>
        ) : (
          <NativeAudioRibbon
            artifactId={artifactId}
            audioTracks={audioTracks}
            enabledTracks={enabledTracks}
            volumes={audioVolumeByStream}
          />
        )}
      </div>

      <PreviewError message={audioError} />
      <PreviewError message={player.error} />
      <PreviewError message={copyError} />
    </section>
  );
}
