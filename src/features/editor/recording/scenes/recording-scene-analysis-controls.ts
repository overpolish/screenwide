// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { screenshotLayout } from "../../screenshot/screenshot-layout";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import {
  createRecordingTimelineEdit,
  RecordingTimelineEdit,
  recordingTimelineOutputToSource,
} from "../../timeline/editing/recording-timeline-edit";
import {
  recordingTimelineLengthMs,
  recordingTimelineMsAt,
} from "../../timeline/editing/recording-timeline-time";
import { retimedSpanClips } from "../recording-overlay-retime";

import { sceneLandingMs } from "./recording-scene-arrangement";
import {
  ownEditedAutoZooms,
  planRecordingAutoZoom,
  visibleScreenArea,
  withAutoZooms,
} from "./recording-scene-auto-zoom";
import { RecordingSceneControls } from "./recording-scene-channel";
import { SceneCamera } from "./recording-scene-placement";
import { recenterSceneScreen } from "./recording-scene-recenter";
import { RecordingSceneClip } from "./recording-scenes";

import type { CaptureSource } from "../../layer-measure";

// The Scene asks that read the recording's picture: the zooms planned from
// the cursor and keys, and Recenter, which finds the screen's content on a
// frame of the scene in hand.

const EMPTY_CLIPS: RecordingSceneClip[] = [];

export function sceneAnalysisControls({
  artifactId,
  camera,
  commitClips,
  getPositionMs,
  inHand,
  latestEdit,
  output,
  screen,
  sourceDurationMs,
}: {
  artifactId: number;
  /** The camera a scene can place: one drawn into the picture. */
  camera: SceneCamera | null;
  /** Commits the clips as they are, auto zooms and all. */
  commitClips: (clips: RecordingSceneClip[]) => void;
  /** Where the playhead is now, in source time. */
  getPositionMs: () => number;
  /** The scene the Scene island acts on, found when asked. */
  inHand: () => RecordingSceneClip | null;
  /** The edit as it stands when a plan lands, which may be after the ask. */
  latestEdit: () => RecordingTimelineEdit;
  output: ScreenshotOutputSettings;
  screen: CaptureSource | undefined;
  sourceDurationMs: number;
}): Pick<
  RecordingSceneControls,
  "autoZoom" | "clearAutoZooms" | "recenterScreen"
> {
  const clipsNow = () => latestEdit().sceneClips ?? EMPTY_CLIPS;
  // The frame Recenter reads: the playhead's where it is in the scene, else
  // the scene's once arrived.
  const frameMsOf = (clip: RecordingSceneClip) => {
    const edit = latestEdit();
    const playheadMs = getPositionMs();
    const atMs = recordingTimelineMsAt(edit, sourceDurationMs, playheadMs);
    if (clip.startMs <= atMs && atMs < clip.endMs) return playheadMs;
    const lengthMs = recordingTimelineLengthMs(edit, sourceDurationMs);
    return (
      recordingTimelineOutputToSource(
        edit,
        sceneLandingMs(clip) / Math.max(1, lengthMs),
      ) * sourceDurationMs
    );
  };
  return {
    autoZoom: () => {
      planRecordingAutoZoom(artifactId, visibleScreenArea(output, screen))
        .then((planned) => {
          // The plan is timed on the whole recording; it lands on the
          // timeline as it now stands.
          const latest = latestEdit();
          commitClips(
            withAutoZooms(
              latest.sceneClips ?? EMPTY_CLIPS,
              retimedSpanClips(planned, {
                next: latest.segments,
                previous: createRecordingTimelineEdit(latest.artifactId)
                  .segments,
                sourceDurationMs,
              }),
            ),
          );
        })
        .catch((cause: unknown) => {
          console.error("Could not make the auto zooms", cause);
        });
    },
    clearAutoZooms: () => {
      commitClips(withAutoZooms(clipsNow(), []));
    },
    recenterScreen: () => {
      const target = inHand();
      if (!target || !screen) return;
      recenterSceneScreen({
        artifactId,
        camera,
        clipId: target.id,
        commit: (next) => {
          commitClips(ownEditedAutoZooms(clipsNow(), next));
        },
        latest: clipsNow,
        layout: screenshotLayout(screen, output),
        output,
        positionMs: frameMsOf(target),
        source: screen,
      });
    },
  };
}
