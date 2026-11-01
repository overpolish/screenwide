// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";

import { useTweenedOutput } from "../../preview/use-tweened-output";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import {
  RecordingTimelinePlaybackRange,
  recordingTimelinePlaybackRanges,
} from "../../timeline/recording-timeline-playback";
import { CameraOverlaySettings } from "../../types";

import { sceneRadius } from "./recording-scene-panel-snapshot";
import { RecordingSceneClip } from "./recording-scenes";
import { sceneClipsBetween } from "./scene-clips-between";

/** Scenes with the timeline they are timed on. */
type TimedScenes = {
  clips: RecordingSceneClip[];
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
  look,
  sessionId,
  sourceDurationMs,
}: {
  clips: RecordingSceneClip[];
  edit: RecordingTimelineEdit;
  look: {
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
    sceneClipsBetween((clip, pane) => sceneRadius(clip, pane, look)),
  );
  useEffect(() => {
    if (sessionId !== null)
      sendScenes(
        queueRef.current,
        {
          clips: drawnClips,
          playbackRanges: recordingTimelinePlaybackRanges(
            edit,
            sourceDurationMs,
          ),
        },
        sessionId,
      );
  }, [drawnClips, edit, sessionId, sourceDurationMs]);
  return (draft: RecordingSceneClip[] | null) => {
    if (sessionId !== null)
      sendScenes(
        queueRef.current,
        {
          clips: draft ?? clips,
          playbackRanges: recordingTimelinePlaybackRanges(
            edit,
            sourceDurationMs,
          ),
        },
        sessionId,
      );
  };
}
