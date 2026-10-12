// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";

import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import {
  RecordingTimelinePlaybackRange,
  recordingTimelinePlaybackRanges,
} from "../../timeline/recording-timeline-playback";

import { RecordingSceneClip } from "./recording-scenes";

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
 * lays out, which it arranges every frame the way the export does. A drag on
 * the lane shows its draft the same way while it lasts, without committing
 * it: the answer takes the draft, or null to go back to the committed clips.
 */
export function useScenePreviewSync({
  clips,
  edit,
  sessionId,
  sourceDurationMs,
}: {
  clips: RecordingSceneClip[];
  edit: RecordingTimelineEdit;
  sessionId: number | null;
  sourceDurationMs: number;
}) {
  const queueRef = useRef<SendQueue>({ busy: false, next: null });
  useEffect(() => {
    if (sessionId !== null)
      sendScenes(
        queueRef.current,
        {
          clips,
          playbackRanges: recordingTimelinePlaybackRanges(
            edit,
            sourceDurationMs,
          ),
        },
        sessionId,
      );
  }, [clips, edit, sessionId, sourceDurationMs]);
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
