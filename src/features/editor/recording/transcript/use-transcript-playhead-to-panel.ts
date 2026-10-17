// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { emitTo } from "@tauri-apps/api/event";
import { useEffect } from "react";

import { toolPanelLabel } from "../../tool-panels/tool-panel-window";
import { EditorKind } from "../../types";

import { RecordingTranscriptPreviewControls } from "./recording-transcript-channel";
import { TRANSCRIPT_PLAYHEAD_EVENT } from "./transcript-api";

/** The playhead of `workspace`'s preview, in source milliseconds, sent to
 * its panel window on its own: once a frame, and only when it moved. */
export function useTranscriptPlayheadToPanel(
  preview: RecordingTranscriptPreviewControls | null,
  workspace: EditorKind,
) {
  useEffect(() => {
    if (!preview) return;
    let lastMs = -1;
    let frame: number | null = null;
    const unsubscribe = preview.subscribe((sourceMs) => {
      if (sourceMs === lastMs) return;
      lastMs = sourceMs;
      if (frame !== null) return;
      frame = requestAnimationFrame(() => {
        frame = null;
        void emitTo(toolPanelLabel(workspace), TRANSCRIPT_PLAYHEAD_EVENT, {
          playheadMs: lastMs,
        }).catch(() => undefined);
      });
    });
    return () => {
      unsubscribe();
      if (frame !== null) cancelAnimationFrame(frame);
    };
  }, [preview, workspace]);
}
