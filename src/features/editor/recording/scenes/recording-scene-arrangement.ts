// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import { RecordingTimelinePlaybackRange } from "../../timeline/recording-timeline-playback";
import { CameraOverlaySettings } from "../../types";

import { towardPlacement } from "./recording-scene-blend";
import { cameraFramingOf, framingWithin } from "./recording-scene-framing";
import { SceneRect } from "./recording-scene-geometry";
import {
  SceneCamera,
  basePlacement,
  targeting,
} from "./recording-scene-placement";
import {
  RecordingSceneClip,
  SceneFraming,
  WHOLE_FRAMING,
  sceneNeedsCamera,
} from "./recording-scenes";

/** How long a scene takes to arrive or leave, in output milliseconds. A clip
 * too short for two of them gives each half of its length. */
const TRANSITION_MS = 600;

export type RecordingSceneArrangement = {
  /** Whether the camera is drawn in front of the screen now; the nearer
   * order while a scene crosses it over. */
  cameraInFront: boolean;
  /** The camera's overlay as the scene draws it, null without a camera. */
  cameraOverlay: CameraOverlaySettings | null;
  /** Where the clip settles the camera's box and the framing it shows there,
   * which a drag on the camera reframes; null without a camera. */
  cameraTarget: { frame: SceneRect; framing: SceneFraming } | null;
  /** The clip under the playhead. */
  clip: RecordingSceneClip;
  /** How opaque the scene draws each pane now, zero where it hides one. */
  opacity: { camera: number; screen: number };
  output: ScreenshotOutputSettings;
  /** The part of the screen the clip shows, held where it fills its box, and
   * the aspect of the crop it frames, which is the picture its focus and zoom
   * are shares of. */
  screen: { aspect: number; framing: SceneFraming };
};

const easeInOutCubic = (value: number) =>
  value < 0.5 ? 4 * value ** 3 : 1 - (-2 * value + 2) ** 3 / 2;

/** Where `sourceMs` lands on the output timeline: within the range holding
 * it, or, cut away, where its source predecessor ends, or at the start of the
 * range earliest in the source. The twin of `output_at_us`. */
const outputAt = (
  ranges: RecordingTimelinePlaybackRange[],
  sourceMs: number,
) => {
  const length = (range: RecordingTimelinePlaybackRange) =>
    (range.sourceEndMs - range.sourceStartMs) / range.playbackRate;
  const starts: number[] = [];
  let outputStartMs = 0;
  for (const range of ranges) {
    if (range.sourceStartMs <= sourceMs && sourceMs < range.sourceEndMs)
      return (
        outputStartMs + (sourceMs - range.sourceStartMs) / range.playbackRate
      );
    starts.push(outputStartMs);
    outputStartMs += length(range);
  }
  let predecessor = -1;
  let earliest = 0;
  ranges.forEach((range, index) => {
    if (
      range.sourceEndMs <= sourceMs &&
      (predecessor < 0 || range.sourceEndMs > ranges[predecessor].sourceEndMs)
    )
      predecessor = index;
    if (range.sourceStartMs < ranges[earliest].sourceStartMs) earliest = index;
  });
  return predecessor >= 0
    ? starts[predecessor] + length(ranges[predecessor])
    : starts[earliest];
};

/**
 * Arranges the screen's `output`, and the camera's overlay where `camera`
 * gives one drawn into the picture, as `clips`, timed on the timeline
 * `ranges` lay out, have them `sourceMs` into the recording; null outside
 * every clip and in a scene that stands idle, where they are left as they
 * are. The twin of `arrange` in `src-tauri/crates/export/src/scenes/arrange.rs`,
 * which draws the frames this places the selection over.
 */
export function arrangedRecordingScene({
  camera,
  clips,
  output,
  ranges,
  sourceMs,
}: {
  camera: SceneCamera | null;
  clips: readonly RecordingSceneClip[];
  output: ScreenshotOutputSettings;
  ranges: RecordingTimelinePlaybackRange[];
  sourceMs: number;
}): RecordingSceneArrangement | null {
  if (output.cropWidth <= 0 || output.cropHeight <= 0 || output.imageWidth <= 0)
    return null;
  const sceneCamera = camera && camera.aspect > 0 ? camera : null;
  const timelineMs =
    ranges.length > 0 ? Math.floor(outputAt(ranges, sourceMs)) : sourceMs;
  const videoMs =
    ranges.length > 0
      ? Math.round(
          ranges.reduce(
            (total, range) =>
              total +
              (range.sourceEndMs - range.sourceStartMs) / range.playbackRate,
            0,
          ),
        )
      : null;
  // A clip at either end of the video, to within a millisecond of rounding,
  // has nothing before it to arrive from or after it to leave for. One that
  // reaches the end holds to it, so the last frame never falls just past it.
  const startsVideo = (clip: RecordingSceneClip) =>
    videoMs !== null && clip.startMs <= 1;
  const endsVideo = (clip: RecordingSceneClip) =>
    videoMs !== null && clip.endMs + 1 >= videoMs;
  const index = clips.findIndex(
    (clip) =>
      clip.startMs <= timelineMs &&
      (timelineMs < clip.endMs || endsVideo(clip)),
  );
  if (index < 0) return null;
  const clip = clips[index];
  const base = basePlacement(output, sceneCamera);
  const targetOf = targeting({
    base,
    camera: sceneCamera,
    canvas: { height: output.height, width: output.width },
  });
  const target = targetOf(clip);
  if (!target) return null;
  const length = clip.endMs - clip.startMs;
  if (!(length > 0)) return null;
  const elapsed = Math.max(0, timelineMs - clip.startMs);
  const window = Math.min(
    TRANSITION_MS,
    startsVideo(clip) || endsVideo(clip) ? length : length / 2,
  );
  const eased = (share: number) =>
    easeInOutCubic(Math.min(1, Math.max(0, share)));
  // A scene hands on to the one butted up against it.
  const previous =
    index > 0 && clips[index - 1].endMs === clip.startMs
      ? clips[index - 1]
      : undefined;
  const next =
    index + 1 < clips.length && clips[index + 1].startMs === clip.endMs
      ? clips[index + 1]
      : undefined;
  const placed =
    !startsVideo(clip) && elapsed < window
      ? towardPlacement(
          (previous && targetOf(previous)) || base,
          target,
          eased(elapsed / window),
        )
      : !endsVideo(clip) &&
          !(next && targetOf(next)) &&
          length - elapsed < window
        ? towardPlacement(base, target, eased((length - elapsed) / window))
        : target;
  const cameraFraming = sceneCamera
    ? (clip.camera ??
      (sceneNeedsCamera(clip)
        ? WHOLE_FRAMING
        : cameraFramingOf(sceneCamera.overlay, sceneCamera.aspect)))
    : null;
  return {
    cameraInFront: placed.cameraFront >= 0.5,
    cameraOverlay: sceneCamera
      ? {
          ...sceneCamera.overlay,
          cameraWidth: placed.camera.width,
          cameraX: placed.camera.x,
          cameraY: placed.camera.y,
          frameHeight: placed.frame.height,
          frameWidth: placed.frame.width,
          frameX: placed.frame.x,
          frameY: placed.frame.y,
          radiusPercent: placed.radius.camera,
        }
      : null,
    cameraTarget:
      sceneCamera && cameraFraming
        ? {
            frame: target.frame,
            framing: framingWithin(
              cameraFraming,
              target.frame,
              sceneCamera.aspect,
            ),
          }
        : null,
    clip,
    opacity: placed.opacity,
    output: {
      ...output,
      cropHeight: placed.screen.height,
      cropWidth: placed.screen.width,
      cropX: placed.screen.x,
      cropY: placed.screen.y,
      imageWidth: placed.image.width,
      imageX: placed.image.x,
      imageY: placed.image.y,
      radiusPercent: placed.radius.screen,
    },
    screen: {
      aspect: output.cropWidth / output.cropHeight,
      framing: framingWithin(
        clip.screen ?? WHOLE_FRAMING,
        target.screen,
        output.cropWidth / output.cropHeight,
      ),
    },
  };
}
