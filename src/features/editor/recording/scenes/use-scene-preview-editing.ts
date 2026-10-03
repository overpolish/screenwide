// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import {
  ScreenshotOutputSettings,
  screenshotOutputDimensions,
} from "../../screenshot/screenshot-output";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { recordingTimelinePlaybackRanges } from "../../timeline/recording-timeline-playback";
import { CameraOverlaySettings, RecordingVideoTrackId } from "../../types";

import { arrangedRecordingScene } from "./recording-scene-arrangement";
import { ownEditedAutoZooms } from "./recording-scene-auto-zoom";
import { pictureRect } from "./recording-scene-custom-crop";
import { reframeWindow } from "./recording-scene-framing";
import { SceneRect } from "./recording-scene-geometry";
import {
  RecordingSceneTarget,
  useRecordingSceneGesture,
} from "./use-recording-scene-gesture";

import type { RecordingCanvasTool } from "../preview/recording-crop-toggle";
import type { RecordingPreviewSelection } from "../use-recording-preview-surface";

/**
 * The scene under the paused preview's playhead, as the preview edits it: the
 * arrangement its panes' outlines and hit areas are placed by, the gesture
 * that edits it, and the crop tool's window on each pane. A preset's pane
 * keeps its shape, the window laid over its unzoomed picture; a custom
 * scene's own boxes are cropped freely, the window over the picture where it
 * sits. The camera only takes part where it is drawn into the picture.
 */
export function useScenePreviewEditing({
  cameraOverlay,
  cameraSource,
  canPreviewBakedCamera,
  canvasTool,
  durationMs,
  edit,
  editGesture,
  onEditChange,
  output,
  positionMs,
}: {
  cameraOverlay: CameraOverlaySettings;
  cameraSource: { height: number; width: number } | undefined;
  canPreviewBakedCamera: boolean;
  canvasTool: RecordingCanvasTool;
  durationMs: number;
  edit: RecordingTimelineEdit | null | undefined;
  editGesture: { beginGesture: () => void; endGesture: () => void };
  /** The screen's composition outside every scene. */
  output: ScreenshotOutputSettings;
  positionMs: number;
  onEditChange?: (edit: RecordingTimelineEdit) => void;
}) {
  const cameraAspect = cameraSource
    ? cameraSource.width / Math.max(1, cameraSource.height)
    : 0;
  const arrangement = useMemo(
    () =>
      edit?.sceneClips
        ? arrangedRecordingScene({
            camera:
              canPreviewBakedCamera && cameraAspect > 0
                ? { aspect: cameraAspect, overlay: cameraOverlay }
                : null,
            clips: edit.sceneClips,
            output,
            ranges: recordingTimelinePlaybackRanges(edit, durationMs),
            sourceMs: positionMs,
          })
        : null,
    [
      cameraAspect,
      cameraOverlay,
      canPreviewBakedCamera,
      durationMs,
      edit,
      output,
      positionMs,
    ],
  );
  const isEditing = canvasTool === "select" || canvasTool === "crop";
  // Each pane as the paused frame draws it: the box it sits in now and the
  // framing it shows there.
  const target: RecordingSceneTarget | null =
    arrangement && edit?.sceneClips && isEditing
      ? {
          // A camera the scene hides has nothing to reframe.
          camera:
            arrangement.cameraOverlay &&
            arrangement.cameraTarget &&
            arrangement.opacity.camera > 0
              ? {
                  aspect: cameraAspect,
                  box: {
                    height: arrangement.cameraOverlay.frameHeight,
                    width: arrangement.cameraOverlay.frameWidth,
                    x: arrangement.cameraOverlay.frameX,
                    y: arrangement.cameraOverlay.frameY,
                  },
                  framing: arrangement.cameraTarget.framing,
                }
              : null,
          clipId: arrangement.clip.id,
          clips: edit.sceneClips,
          screen: {
            ...arrangement.screen,
            box: {
              height: arrangement.output.cropHeight,
              width: arrangement.output.cropWidth,
              x: arrangement.output.cropX,
              y: arrangement.output.cropY,
            },
          },
        }
      : null;
  const gesture = useRecordingSceneGesture({
    editGesture,
    // A zoom reframed on the canvas is yours from then on.
    onClipsChange: (clips) => {
      if (edit)
        onEditChange?.({
          ...edit,
          sceneClips: ownEditedAutoZooms(edit.sceneClips ?? [], clips),
        });
    },
    primaryOutput: output,
    target,
  });
  // The crop tool's windows, as shares of the canvas the native overlay
  // places selections in.
  const canvas = screenshotOutputDimensions(output);
  const share = (rect: SceneRect) => ({
    height: rect.height / Math.max(1, canvas.height),
    width: rect.width / Math.max(1, canvas.width),
    x: rect.x / Math.max(1, canvas.width),
    y: rect.y / Math.max(1, canvas.height),
  });
  const boxes = arrangement?.clip.boxes;
  const cropWindow = (
    pane: NonNullable<RecordingSceneTarget["camera"]>,
    { inPlace, layerId }: { inPlace: boolean; layerId: number },
  ): RecordingPreviewSelection => {
    const { image, window } = inPlace
      ? { image: pictureRect(pane), window: pane.box }
      : reframeWindow(pane);
    return {
      cropMode: true,
      framed: !inPlace,
      image: share(image),
      layerId,
      paneIndex: canPreviewBakedCamera ? 0 : layerId,
      radiusPercent: 0,
      rect: share(window),
    };
  };
  const reframeSelections: Partial<
    Record<RecordingVideoTrackId, RecordingPreviewSelection>
  > | null =
    target && canvasTool === "crop"
      ? {
          primary: cropWindow(target.screen, {
            inPlace: Boolean(boxes),
            layerId: 0,
          }),
          ...(target.camera
            ? {
                camera: cropWindow(target.camera, {
                  inPlace: Boolean(boxes?.camera),
                  layerId: 1,
                }),
              }
            : {}),
        }
      : null;
  return {
    arrangement,
    gesture,
    /** Whether the scene at the playhead is custom, its panes' boxes moved
     * and resized by the select tool rather than fixed. */
    isCustom: Boolean(arrangement?.clip.boxes),
    /** Whether a scene places the panes at the playhead. */
    isFramed: arrangement !== null,
    reframeSelections,
  };
}
