// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { useEffect } from "react";

import type { CropTurn } from "../../preview/use-crop-turn";

type ScenePane = "camera" | "screen" | null;

const sendSceneCrop = (
  sessionId: number,
  pane: ScenePane,
  screenTurn: number,
) =>
  invoke<null>("set_recording_preview_scene_reframe", {
    pane,
    screenTurn,
    sessionId,
  }).catch((cause: unknown) => {
    console.error("Could not show the scene's picture whole", cause);
  });

/**
 * Tells the native preview which pane of the scene under the playhead the
 * crop tool is reframing, so it draws that pane unzoomed beneath the crop
 * window, and how much of the scene's turn of the screen to draw as the
 * selection's `cropTurn` eases it out before the reframe and back in after.
 * Both live in the session's sources, which stopping the session drops, so
 * nothing is sent to let them go: the hook only goes with its session, and a
 * reset sent then reached a session already stopped (as an export's
 * suspension stops it) and failed.
 */
export function useSceneReframeDisplay(
  sessionId: number | null,
  { cropTurn, reframePane }: { cropTurn: CropTurn; reframePane: ScenePane },
) {
  const screenTurn = cropTurn.share;
  useEffect(() => {
    if (sessionId === null) return;
    void sendSceneCrop(sessionId, reframePane, screenTurn);
  }, [reframePane, screenTurn, sessionId]);
}
