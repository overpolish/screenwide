// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";

import { canvasShortest } from "../../layer-measure";
import { useTweenedOutput } from "../../preview/use-tweened-output";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { recordingTimelineGaps } from "../../timeline/recording-strip/recording-timeline-gaps";
import {
  RecordingTimelinePlaybackRange,
  recordingTimelinePlaybackRanges,
} from "../../timeline/recording-timeline-playback";
import { CameraOverlaySettings } from "../../types";

import {
  sceneCameraRadius,
  scenePaneShortest,
  sceneScreenRadius,
} from "./recording-scene-panel-snapshot";
import { SceneCamera } from "./recording-scene-placement";
import { RecordingSceneClip } from "./recording-scenes";
import { sceneClipsBetween } from "./scene-clips-between";

import type { GapScene } from "../../../../bindings/GapScene";

/** Scenes with the timeline they are timed on, the gaps drawn among them,
 * and the scene the crop tool holds where its panes stood, which the preview
 * draws only while the crop tool is on a pane. */
type TimedScenes = {
  clips: RecordingSceneClip[];
  gaps: GapScene[];
  held: RecordingSceneClip | null;
  playbackRanges: RecordingTimelinePlaybackRange[];
};

/** The scenes waiting to reach the preview, and whether a send is out. */
type SendQueue = { busy: boolean; next: TimedScenes | null };

/** Sends `scenes` to session `sessionId`'s preview, one send at a time: what
 * arrives while one is out waits, and only the newest is sent after it, so a
 * drag sends no more than the preview can draw. The clips and their timeline
 * travel together, so no frame arranges clips on a timeline they were not
 * timed for. */
const sendScenes = (
  queue: SendQueue,
  scenes: TimedScenes,
  sessionId: number,
) => {
  queue.next = scenes;
  if (queue.busy) return;
  const flush = () => {
    const next = queue.next;
    queue.busy = next !== null;
    if (!next) return;
    queue.next = null;
    void invoke<null>("set_recording_preview_scenes", {
      ...next,
      sessionId,
    })
      .catch((cause: unknown) => {
        console.error("Could not send the scenes to the preview", cause);
      })
      .finally(flush);
  };
  flush();
};

/**
 * Keeps the native preview drawing `clips`, timed on the timeline `edit`
 * lays out, which it arranges every frame the way the export does. A scene's
 * setting chosen on an island is drawn moving to it (`picture-tween.ts`),
 * the corners of a pane a scene starts or stops rounding moving from or to
 * the recording's own, which `look` holds. A drag on the lane shows its
 * draft the same way while it lasts, without committing it: the answer
 * takes the draft, or null to go back to the committed clips.
 */
export function useScenePreviewSync({
  clips,
  edit,
  held,
  look,
  sessionId,
  sourceDurationMs,
}: {
  clips: RecordingSceneClip[];
  edit: RecordingTimelineEdit;
  /** The scene the crop tool holds where its panes stood. */
  held: RecordingSceneClip | null;
  look: {
    /** The camera a scene can place: one drawn into the picture. */
    camera: SceneCamera | null;
    cameraOverlay: CameraOverlaySettings;
    output: ScreenshotOutputSettings;
  };
  sessionId: number | null;
  sourceDurationMs: number;
}) {
  const queueRef = useRef<SendQueue>({ busy: false, next: null });
  const drawnClips = useTweenedOutput(
    clips,
    { part: "scenes", workspace: "recording" },
    sceneClipsBetween({
      camera: (clip) => sceneCameraRadius(clip, look.cameraOverlay),
      canvas: canvasShortest(look.output),
      frames: (clip) => scenePaneShortest(clip, look),
      screen: (clip) => sceneScreenRadius(clip, look.output),
    }),
  );
  const timed = (sent: RecordingSceneClip[]): TimedScenes => ({
    clips: sent,
    gaps: recordingTimelineGaps(edit, sourceDurationMs),
    held,
    playbackRanges: recordingTimelinePlaybackRanges(edit, sourceDurationMs),
  });
  useEffect(() => {
    if (sessionId !== null)
      sendScenes(queueRef.current, timed(drawnClips), sessionId);
    // eslint-disable-next-line @eslint-react/exhaustive-deps -- `timed` reads only `edit`, `held` and `sourceDurationMs`.
  }, [drawnClips, edit, held, sessionId, sourceDurationMs]);
  return (draft: RecordingSceneClip[] | null) => {
    if (sessionId !== null)
      sendScenes(queueRef.current, timed(draft ?? clips), sessionId);
  };
}
