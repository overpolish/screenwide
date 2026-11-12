// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject } from "react";

import { t } from "../../../../i18n/i18n";
import { CircularProgress } from "../../../../primitives/circular-progress/circular-progress";
import {
  RecordingOutputSettings,
  defaultScreenshotOutput,
} from "../../screenshot/screenshot-output";
import { NativeAudioRibbon } from "../../timeline/audio/native-audio-ribbon";
import { useRecordingPreviewTracks } from "../../timeline/tracks/use-recording-preview-tracks";

import { NativeRecordingWorkspaceViewport } from "./native-recording-workspace-viewport";
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
    <p className="px-inset pb-inset-sm text-caption text-error">{message}</p>
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
    screenCanvasRef: RefObject<HTMLCanvasElement | null>;
    activeRecordingOutput?: RecordingOutputSettings;
    copyError?: string | null;
  }) {
  const isPreparing =
    previewLayout === undefined && (player.isPreparing || isPreparingPreview);
  const shownLayout =
    visibleLayout && visibleLayout.panes.length > 0 ? visibleLayout : null;
  // Every arrangement, the audio ribbon too, shares one viewport element.
  // Swapping One video for Separate files, or the screen for the ribbon,
  // would otherwise mount a fresh viewport at zero size, and the native
  // layout reads unsized markers as a recording with nothing to show: it
  // hides the picture until the new viewport has measured itself.
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
        }
      : shownLayout && activeRecordingOutput
        ? {
            ...recordingOutputWorkspace(
              visiblePaneEntries,
              activeRecordingOutput,
            ),
            isBusy: false,
          }
        : shownLayout
          ? {
              ...recordingLayoutWorkspace(shownLayout, visibleCanvasRefs),
              isBusy: isPreparing,
            }
          : null;
  return (
    <section className="relative flex min-h-0 min-w-0 grow flex-col">
      {/* The picture meets the tool column, the title bar and the strip on
          rounded corners; the viewport takes them up, and the native
          drawables are clipped to them. */}
      <div className="flex min-h-0 grow items-stretch justify-center rounded-ss-panel rounded-es-panel">
        {!layout ? (
          <div className="flex grow items-center justify-center gap-inline text-caption text-fg-secondary">
            <CircularProgress
              aria-label={t("editor-recording-preparing-recording")}
              isIndeterminate
            />
            {t("editor-recording-preparing-recording")}
          </div>
        ) : (
          <div className="flex min-h-0 min-w-0 grow flex-col rounded-[inherit]">
            <NativeRecordingWorkspaceViewport
              {...(workspace ?? {
                ariaLabel: t("editor-timeline-audio-visualizer"),
                audioOnly: true,
                isBusy: false,
                panes: [],
                workspaceHeight: 0,
                workspaceWidth: 0,
              })}
            />
            {workspace ? null : (
              <NativeAudioRibbon
                artifactId={artifactId}
                audioTracks={audioTracks}
                enabledTracks={enabledTracks}
                volumes={audioVolumeByStream}
              />
            )}
          </div>
        )}
      </div>

      <PreviewError message={audioError} />
      <PreviewError message={player.error} />
      <PreviewError message={copyError} />
    </section>
  );
}
