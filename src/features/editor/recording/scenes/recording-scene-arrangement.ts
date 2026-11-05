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
  SceneRotation,
  WHOLE_FRAMING,
  sceneNeedsCamera,
} from "./recording-scenes";

/** How long a scene takes to arrive or leave, in output milliseconds. A clip
 * too short for two of them gives each half of its length. The twin of
 * `SCENE_TRANSITION_MS` in `scene_clock.rs`. */
export const TRANSITION_MS = 600;

/** Where choosing `clip` brings the playhead, in output ms: once the scene
 * has arrived, so the picture shows it settled rather than on its way. A
 * clip too short for two arrivals lands at its middle, and one starting the
 * video, which arrives at once, at its start. */
export const sceneLandingMs = (clip: { endMs: number; startMs: number }) =>
  clip.startMs <= 1
    ? clip.startMs
    : clip.startMs +
      Math.max(0, Math.min(TRANSITION_MS, (clip.endMs - clip.startMs) / 2));

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
  /** The screen's output as the scene draws it, its crop padded outwards by
   * the scene's inset as `arrange.rs` pads it, so the selection and the hit
   * targets lie on the padded box the preview draws. */
  output: ScreenshotOutputSettings;
  /** How the scene turns the screen in depth now. */
  rotation: SceneRotation;
  /** The part of the screen the clip shows, held where it fills its box, the
   * box itself without the scene's padding, and the aspect of the crop it
   * frames, which is the picture its focus and zoom are shares of. */
  screen: { aspect: number; box: SceneRect; framing: SceneFraming };
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
  // A scene that pads its screen grows the crop outwards past its box, as the
  // recording's own inset pads its crop.
  const inset = Math.max(0, placed.inset);
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
      cropHeight: placed.screen.height + inset * 2,
      cropWidth: placed.screen.width + inset * 2,
      cropX: placed.screen.x - inset,
      cropY: placed.screen.y - inset,
      imageWidth: placed.image.width,
      imageX: placed.image.x,
      imageY: placed.image.y,
      radiusPercent: placed.radius.screen,
    },
    rotation: placed.rotation,
    screen: {
      aspect: output.cropWidth / output.cropHeight,
      box: placed.screen,
      framing: framingWithin(
        clip.screen ?? WHOLE_FRAMING,
        target.screen,
        output.cropWidth / output.cropHeight,
      ),
    },
  };
}
