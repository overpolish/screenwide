// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useCallback } from "react";

import { RecordingCanvasTool } from "../../recording/preview/recording-crop-toggle";
import {
  previewViewport,
  useToolPanel,
} from "../../tool-panels/use-tool-panel";
import { RecordingTrackId } from "../../types";

export function useRecordingTrackSelection({
  clearAnnotations,
  clearOverlays,
  onSelectedTrackChange,
  setTool,
}: {
  clearAnnotations: RefObject<() => void>;
  /** Lets go of the shortcuts and captions chosen in their lanes. */
  clearOverlays: () => void;
  setTool: (tool: RecordingCanvasTool) => void;
  onSelectedTrackChange?: (trackId: RecordingTrackId | null) => void;
}) {
  const { openPanel } = useToolPanel("recording");
  return useCallback(
    (trackId: RecordingTrackId) => {
      clearAnnotations.current();
      clearOverlays();
      onSelectedTrackChange?.(trackId);
      setTool("select");
      const bounds = previewViewport()?.getBoundingClientRect();
      if (bounds) void openPanel("selection", bounds, false);
    },
    [
      clearAnnotations,
      clearOverlays,
      onSelectedTrackChange,
      openPanel,
      setTool,
    ],
  );
}
