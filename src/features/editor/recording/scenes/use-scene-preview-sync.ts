// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef } from "react";

import { RecordingSceneClip } from "./recording-scenes";

/** The scenes waiting to reach the preview, and whether a send is out. */
type SendQueue = { busy: boolean; next: RecordingSceneClip[] | null };

/** Sends `clips` to session `sessionId`'s preview, one send at a time: what
 * arrives while one is out waits, and only the newest is sent after it, so a
 * drag sends no more than the preview can draw. */
const sendScenes = (
  queue: SendQueue,
  { clips, sessionId }: { clips: RecordingSceneClip[]; sessionId: number },
) => {
  queue.next = clips;
  if (queue.busy) return;
  const flush = () => {
    const next = queue.next;
    queue.busy = next !== null;
    if (!next) return;
    queue.next = null;
    void invoke<null>("set_recording_preview_scenes", {
      clips: next,
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
 * Keeps the native preview drawing `clips`, which it arranges every frame
 * from the way the export does. A drag on the lane shows its draft the same
 * way while it lasts, without committing it: the answer takes the draft, or
 * null to go back to the committed clips.
 */
export function useScenePreviewSync({
  clips,
  sessionId,
}: {
  clips: RecordingSceneClip[];
  sessionId: number | null;
}) {
  const queueRef = useRef<SendQueue>({ busy: false, next: null });
  useEffect(() => {
    if (sessionId !== null) sendScenes(queueRef.current, { clips, sessionId });
  }, [clips, sessionId]);
  return (draft: RecordingSceneClip[] | null) => {
    if (sessionId !== null)
      sendScenes(queueRef.current, { clips: draft ?? clips, sessionId });
  };
}
