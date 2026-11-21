// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo } from "react";

import {
  ScreenshotOutputSettings,
  screenshotOutputDimensions,
} from "../../screenshot/screenshot-output";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { recordingTimelinePlaybackRanges } from "../../timeline/recording-timeline-playback";
import { CameraOverlaySettings, RecordingVideoTrackId } from "../../types";
import { faceAt, useFacePath } from "../preview/face-path";

import { arrangedRecordingScene } from "./recording-scene-arrangement";
import { ownEditedAutoZooms } from "./recording-scene-auto-zoom";
import {
  releaseSceneCrop,
  useSceneCropHold,
  withHeldScene,
} from "./recording-scene-crop-hold";
import { pictureRect } from "./recording-scene-custom-crop";
import { drawnCameraRect, reframeWindow } from "./recording-scene-framing";
import { SceneRect } from "./recording-scene-geometry";
import { sceneNeedsCamera, sceneShowsScreen } from "./recording-scenes";
import {
  RecordingSceneTarget,
  useRecordingSceneGesture,
} from "./use-recording-scene-gesture";

import type { CaptureSource } from "../../layer-measure";
import type { RecordingCanvasTool } from "../preview/recording-canvas-tool";
import type { RecordingPreviewSelection } from "../use-recording-preview-surface";

/** A crop window over a scene's pane, and the box it settles into once the
 * crop has closed, which `openingCropChrome` eases it toward. */
type SceneCropWindow = RecordingPreviewSelection & {
  settled?: { height: number; width: number; x: number; y: number };
};

/**
 * The scene under the paused preview's playhead, as the preview edits it: the
 * arrangement its panes' outlines and hit areas are placed by, the gesture
 * that edits it, and the crop tool's window on each pane. The screen's crop
 * is each scene's own and free: its box is the window, over the picture
 * where it sits, a preset's made custom as it is cropped. A preset's camera
 * keeps its box's shape, the window laid over its unzoomed picture; a custom
 * scene's camera box is cropped freely as the screen is. The camera only
 * takes part where it is drawn into the picture.
 */
export function useScenePreviewEditing({
  cameraOverlay,
  canPreviewBakedCamera,
  canvasTool,
  captures,
  durationMs,
  edit,
  editGesture,
  onEditChange,
  output,
  positionMs,
}: {
  cameraOverlay: CameraOverlaySettings;
  canPreviewBakedCamera: boolean;
  canvasTool: RecordingCanvasTool;
  /** Each picture as captured; none for one the recording does not have. */
  captures: Partial<Record<RecordingVideoTrackId, CaptureSource>>;
  durationMs: number;
  edit: RecordingTimelineEdit | null | undefined;
  editGesture: { beginGesture: () => void; endGesture: () => void };
  /** The screen's composition outside every scene. */
  output: ScreenshotOutputSettings;
  positionMs: number;
  onEditChange?: (edit: RecordingTimelineEdit) => void;
}) {
  const cameraSource = captures.camera;
  const screenSource = captures.primary;
  const cameraAspect = cameraSource
    ? cameraSource.width / Math.max(1, cameraSource.height)
    : 0;
  // A camera following the face is placed by it, as the preview places it.
  const facePath = useFacePath();
  const face = useMemo(
    () => (facePath ? faceAt(facePath, positionMs) : null),
    [facePath, positionMs],
  );
  const camera = useMemo(
    () =>
      canPreviewBakedCamera && cameraAspect > 0
        ? { aspect: cameraAspect, face, overlay: cameraOverlay }
        : null,
    [cameraAspect, cameraOverlay, canPreviewBakedCamera, face],
  );
  // Drawn as the preview draws them: a preset the crop tool is on held where
  // its panes stood, until the crop has closed after the tool is put away.
  const held = useSceneCropHold(edit?.sceneClips);
  const drawnClips = useMemo(
    () => edit?.sceneClips && withHeldScene(edit.sceneClips, held),
    [edit?.sceneClips, held],
  );
  const arrangement = useMemo(
    () =>
      edit && drawnClips && screenSource
        ? arrangedRecordingScene({
            camera,
            clips: drawnClips,
            output,
            ranges: recordingTimelinePlaybackRanges(edit, durationMs),
            screen: screenSource,
            sourceMs: positionMs,
          })
        : null,
    [camera, drawnClips, durationMs, edit, output, positionMs, screenSource],
  );
  // Where the panes settle once the crop has closed: a preset held while
  // cropped is laid out at its new shapes then, which the crop's window
  // eases toward as the picture closes onto it.
  const settled = useMemo(
    () =>
      held && edit?.sceneClips && screenSource
        ? arrangedRecordingScene({
            camera,
            clips: edit.sceneClips,
            output,
            ranges: recordingTimelinePlaybackRanges(edit, durationMs),
            screen: screenSource,
            sourceMs: positionMs,
          })
        : arrangement,
    [
      arrangement,
      camera,
      durationMs,
      edit,
      held,
      output,
      positionMs,
      screenSource,
    ],
  );
  // A preset held while cropped is laid out at its new shapes once the crop
  // has closed (`useRecordingCropTurn`) or the playhead leaves its scene.
  const sceneId = arrangement?.clip.id ?? null;
  useEffect(() => releaseSceneCrop, [sceneId]);
  const isEditing = canvasTool === "select" || canvasTool === "crop";
  const cameraFollows =
    (arrangement?.clip.followFace ?? cameraOverlay.followFace) === true;
  // Each pane as the paused frame draws it: the box it sits in now, the
  // framing it shows there as drawn and, for a camera following the face,
  // where its framing has the face sit, which a crop keeps.
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
                  framing: arrangement.cameraTarget.picture,
                }
              : null,
          cameraFaceSpot: arrangement.cameraTarget?.framing ?? null,
          clipId: arrangement.clip.id,
          clips: edit.sceneClips,
          screen: arrangement.screen,
          screenInset: Math.max(
            0,
            (arrangement.output.cropWidth - arrangement.screen.box.width) / 2,
          ),
        }
      : null;
  const gesture = useRecordingSceneGesture({
    camera,
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
  const settledBox = (layerId: number): SceneRect | null => {
    const overlay = settled?.cameraOverlay;
    if (layerId === 0) return settled?.screen.box ?? null;
    return overlay
      ? {
          height: overlay.frameHeight,
          width: overlay.frameWidth,
          x: overlay.frameX,
          y: overlay.frameY,
        }
      : null;
  };
  const cropWindow = (
    pane: NonNullable<RecordingSceneTarget["camera"]>,
    { inPlace, layerId }: { inPlace: boolean; layerId: number },
  ): SceneCropWindow => {
    const { image, window } = inPlace
      ? { image: pictureRect(pane), window: pane.box }
      : reframeWindow(pane);
    const settledWindow = inPlace ? settledBox(layerId) : null;
    return {
      // A camera following the face holds it in the middle of the crop.
      cropAnchored: layerId === 1 && cameraFollows,
      cropMode: true,
      framed: !inPlace,
      // The shade lies where the preview draws the camera's picture.
      image: share(
        layerId === 1 && canPreviewBakedCamera ? drawnCameraRect(image) : image,
      ),
      layerId,
      paneIndex: canPreviewBakedCamera ? 0 : layerId,
      radiusPercent: 0,
      rect: share(window),
      ...(settledWindow ? { settled: share(settledWindow) } : {}),
    };
  };
  const reframeSelections: Partial<
    Record<RecordingVideoTrackId, SceneCropWindow>
  > | null = target
    ? {
        primary: cropWindow(target.screen, {
          inPlace: true,
          layerId: 0,
        }),
        ...(target.camera
          ? {
              camera: cropWindow(target.camera, {
                inPlace: !boxes || Boolean(boxes.camera),
                layerId: 1,
              }),
            }
          : {}),
      }
    : null;
  const preset =
    arrangement && !arrangement.clip.boxes ? arrangement.clip.preset : null;
  return {
    arrangement,
    /** Whether the scene at the playhead is a preset that shows the camera,
     * whose camera the select tool moves and resizes as a custom scene's,
     * making it custom (`scene_pane_gesture.rs`), as it does the screen. */
    freesCamera:
      preset !== null && (preset === "full" || sceneNeedsCamera({ preset })),
    /** Whether the scene at the playhead is a preset that shows the screen,
     * whose screen the select tool moves and resizes as a custom scene's,
     * making it custom (`scene_pane_gesture.rs`); camera only and blank
     * hide it. */
    freesScreen: preset !== null && sceneShowsScreen({ preset }),
    gesture,
    /** Whether the scene at the playhead is custom, its panes' boxes moved
     * and resized by the select tool rather than fixed. */
    isCustom: Boolean(arrangement?.clip.boxes),
    /** Whether a scene places the panes at the playhead. */
    isFramed: arrangement !== null,
    reframeSelections,
  };
}
