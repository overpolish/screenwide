// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject } from "react";

import { CircularProgress } from "../../../components/base/circular-progress/circular-progress";
import { Text } from "../../../components/base/text/text";
import {
  RecordingOutputSettings,
  defaultScreenshotOutput,
} from "../screenshot-output";

import { BakedCameraPreviewViewport } from "./baked-camera-preview-viewport";
import { NativeAudioRibbon } from "./native-audio-ribbon";
import { RecordingCanvasTool } from "./recording-crop-toggle";
import { RecordingOutputPreviewViewport } from "./recording-output-preview-viewport";
import { RecordingPreviewViewport } from "./recording-preview-viewport";
import { useRecordingPreviewTracks } from "./use-recording-preview-tracks";
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

/** The picture above the timeline: whichever viewport the tracks on screen ask
 * for, with the audio ribbon standing in when there is no video at all. */
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
  return (
    <section className="relative flex min-h-0 min-w-0 grow flex-col">
      <div className="flex min-h-0 grow items-stretch justify-center">
        {!layout ? (
          <div className="flex grow items-center justify-center gap-3 text-xs text-muted">
            <CircularProgress
              aria-label="Preparing recording preview"
              isIndeterminate
            />
            Preparing recording preview
          </div>
        ) : canPreviewBakedCamera && screenPane && cameraPane ? (
          <div className="flex min-h-0 min-w-0 grow flex-col">
            <BakedCameraPreviewViewport
              isBusy={
                previewLayout === undefined &&
                (player.isPreparing || isPreparingPreview)
              }
              outputSettings={
                // Draft output keeps the frame and workspace on the resize
                // before it reaches the editor window's state.
                activeRecordingOutput?.primary ??
                defaultScreenshotOutput(screenPane.width, screenPane.height)
              }
              screenCanvasRef={screenCanvasRef}
              tool={canvasTool}
            />
          </div>
        ) : visibleLayout &&
          visibleLayout.panes.length > 0 &&
          activeRecordingOutput ? (
          <div className="flex min-h-0 min-w-0 grow flex-col">
            <RecordingOutputPreviewViewport
              entries={visiblePaneEntries}
              outputs={activeRecordingOutput}
              tool={canvasTool}
            />
          </div>
        ) : visibleLayout && visibleLayout.panes.length > 0 ? (
          <div className="flex min-h-0 min-w-0 grow flex-col">
            <RecordingPreviewViewport
              canvasRefs={visibleCanvasRefs}
              isBusy={
                previewLayout === undefined &&
                (player.isPreparing || isPreparingPreview)
              }
              layout={visibleLayout}
            />
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
