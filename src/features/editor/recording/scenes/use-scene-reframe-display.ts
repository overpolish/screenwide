// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { useEffect } from "react";

import type { CropTurn } from "../../preview/use-crop-turn";

type ScenePane = "camera" | "screen" | null;

type SceneCropAsk = {
  open: number;
  pane: ScenePane;
  screenTurn: number;
  sessionId: number;
};

/** The ask waiting to reach the preview, and whether a send is out: an ease
 * asks every frame, and only the newest is sent once the one out lands, so
 * an older step never arrives after a newer one. */
const queue: { busy: boolean; next: SceneCropAsk | null } = {
  busy: false,
  next: null,
};

const sendSceneCrop = (ask: SceneCropAsk) => {
  queue.next = ask;
  if (queue.busy) return;
  const flush = () => {
    const next = queue.next;
    queue.busy = next !== null;
    if (!next) return;
    queue.next = null;
    void invoke<null>("set_recording_preview_scene_reframe", next)
      .catch((cause: unknown) => {
        console.error("Could not show the scene's picture whole", cause);
      })
      .finally(flush);
  };
  flush();
};

/**
 * Tells the native preview which pane of the scene under the playhead the
 * crop tool opens out, how far it is open as the selection's `cropTurn`
 * eases it out and back, and how much of the scene's turn of the screen to
 * draw meanwhile. All live in the session's sources, which stopping the
 * session drops, so nothing is sent to let them go: the hook only goes with
 * its session, and a reset sent then reached a session already stopped (as
 * an export's suspension stops it) and failed.
 */
export function useSceneReframeDisplay(
  sessionId: number | null,
  { cropTurn, reframePane }: { cropTurn: CropTurn; reframePane: ScenePane },
) {
  const { open, share: screenTurn } = cropTurn;
  useEffect(() => {
    if (sessionId === null) return;
    sendSceneCrop({ open, pane: reframePane, screenTurn, sessionId });
  }, [open, reframePane, screenTurn, sessionId]);
}
