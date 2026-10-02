// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { useEffect } from "react";

/**
 * Tells the native preview which pane of the scene under the playhead the
 * crop tool is reframing, so it draws that pane unzoomed beneath the crop
 * window, and that none is once the tool or the scene is let go.
 */
export function useSceneReframeDisplay(
  sessionId: number | null,
  pane: "camera" | "screen" | null,
) {
  useEffect(() => {
    if (sessionId === null) return;
    const send = (next: "camera" | "screen" | null) =>
      invoke<null>("set_recording_preview_scene_reframe", {
        pane: next,
        sessionId,
      }).catch((cause: unknown) => {
        console.error("Could not show the scene's picture whole", cause);
      });
    void send(pane);
    return () => {
      if (pane !== null) void send(null);
    };
  }, [pane, sessionId]);
}
