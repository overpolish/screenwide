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

/** Where `sourceMs` lands on the output timeline, a moment cut away landing on
 * the join the cut left. The twin of `output_at_us`. */
const outputAt = (
  ranges: RecordingTimelinePlaybackRange[],
  sourceMs: number,
) => {
  let outputStartMs = 0;
  for (const range of ranges) {
    if (sourceMs < range.sourceStartMs) return outputStartMs;
    if (sourceMs < range.sourceEndMs)
      return (
        outputStartMs + (sourceMs - range.sourceStartMs) / range.playbackRate
      );
    outputStartMs +=
      (range.sourceEndMs - range.sourceStartMs) / range.playbackRate;
  }
  return ranges.length > 0 ? outputStartMs : sourceMs;
};

/**
 * Arranges the screen's `output`, and the camera's overlay where `camera`
 * gives one drawn into the picture, as `clips` have them `sourceMs` into the
 * recording, timed on `ranges`; null outside every clip and in a scene that
 * stands idle, where they are left as they are. The twin of `arrange` in
 * `src-tauri/src/editor/scenes/arrange.rs`, which draws the frames this
 * places the selection over.
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
  const videoEndMs =
    ranges.length > 0 ? ranges[ranges.length - 1].sourceEndMs : null;
  // The kept ranges bound the video, to within a millisecond of rounding. A
  // clip that reaches its end holds to it, so the last frame never falls just
  // past the clip.
  const reachesEnd = (clip: RecordingSceneClip) =>
    videoEndMs !== null && clip.endMs + 1 >= videoEndMs;
  const index = clips.findIndex(
    (clip) =>
      clip.startMs <= sourceMs && (sourceMs < clip.endMs || reachesEnd(clip)),
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
  const from = outputAt(ranges, clip.startMs);
  const length = outputAt(ranges, clip.endMs) - from;
  if (!(length > 0)) return null;
  const elapsed = Math.max(0, outputAt(ranges, sourceMs) - from);
  // A scene at either end of the video holds there: there is nothing before
  // it to arrive from, or after it to leave for. Its other transition may then
  // take the whole clip.
  const startsVideo =
    ranges.length > 0 && clip.startMs <= ranges[0].sourceStartMs + 1;
  const endsVideo = reachesEnd(clip);
  const window = Math.min(
    TRANSITION_MS,
    startsVideo || endsVideo ? length : length / 2,
  );
  const eased = (share: number) =>
    easeInOutCubic(Math.min(1, Math.max(0, share)));
  const previous = index > 0 ? clips[index - 1] : undefined;
  const next = index + 1 < clips.length ? clips[index + 1] : undefined;
  const placed =
    !startsVideo && elapsed < window
      ? towardPlacement(
          (previous?.endMs === clip.startMs && targetOf(previous)) || base,
          target,
          eased(elapsed / window),
        )
      : !endsVideo &&
          !(next?.startMs === clip.endMs && targetOf(next)) &&
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
