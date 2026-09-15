// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useCallback } from "react";

import { previewViewport, useToolPanel } from "../tool-panels/use-tool-panel";
import { RecordingTrackId } from "../types";

import { RecordingCanvasTool } from "./recording-crop-toggle";

export function useRecordingTrackSelection({
  clearAnnotations,
  clearKeyboard,
  onSelectedTrackChange,
  setTool,
}: {
  clearAnnotations: RefObject<() => void>;
  clearKeyboard: () => void;
  setTool: (tool: RecordingCanvasTool) => void;
  onSelectedTrackChange?: (trackId: RecordingTrackId | null) => void;
}) {
  const { openPanel } = useToolPanel("recording");
  return useCallback(
    (trackId: RecordingTrackId) => {
      clearAnnotations.current();
      clearKeyboard();
      onSelectedTrackChange?.(trackId);
      setTool("select");
      const bounds = previewViewport()?.getBoundingClientRect();
      if (bounds) void openPanel("selection", bounds, false);
    },
    [
      clearAnnotations,
      clearKeyboard,
      onSelectedTrackChange,
      openPanel,
      setTool,
    ],
  );
}
