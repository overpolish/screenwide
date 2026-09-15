// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useCallback, useEffect, useMemo, useRef } from "react";

import { RecordingTrackId, RecordingVideoTrackId } from "../types";

import { RecordingCanvasTool } from "./recording-crop-toggle";
import { useRecordingPreviewSelection } from "./use-recording-preview-selection";

/** Picking a canvas tool up and putting it down, and what each change clears
 * on its way in. */
export function useRecordingPreviewCanvasTool({
  activeVideoTrack,
  bakeCamera,
  canvasTool,
  clearAnnotationRef,
  hasVisiblePanes,
  keyboardTimeline,
  onSelectedTrackChange,
  setCanvasTool,
}: {
  activeVideoTrack: RecordingVideoTrackId | null;
  bakeCamera: boolean;
  canvasTool: RecordingCanvasTool;
  clearAnnotationRef: RefObject<() => void>;
  hasVisiblePanes: boolean;
  keyboardTimeline: ReturnType<
    typeof useRecordingPreviewSelection
  >["keyboardTimeline"];
  setCanvasTool: (tool: RecordingCanvasTool) => void;
  onSelectedTrackChange?: (trackId: RecordingTrackId | null) => void;
}) {
  const canvasToolRef = useRef(canvasTool);
  canvasToolRef.current = canvasTool;
  const changeCanvasTool = useCallback(
    (next: RecordingCanvasTool) => {
      if (next === "arrow") {
        keyboardTimeline.selection.onClear();
        onSelectedTrackChange?.(
          bakeCamera ? "primary" : (activeVideoTrack ?? "primary"),
        );
      } else if (next !== "select") clearAnnotationRef.current();
      setCanvasTool(next);
    },
    [
      setCanvasTool,
      keyboardTimeline.selection,
      onSelectedTrackChange,
      bakeCamera,
      activeVideoTrack,
      clearAnnotationRef,
    ],
  );
  // A canvas tool acts on the panes on screen. When the last video track is
  // switched off there is nothing left for it to act on, so the tool is put
  // down and its panel goes with it rather than staying up over the ribbon
  // saying nothing is selected.
  useEffect(() => {
    if (!hasVisiblePanes && canvasToolRef.current !== null) setCanvasTool(null);
  }, [hasVisiblePanes, setCanvasTool]);
  // One toggle apiece, built from one function: picking up the tool already in
  // hand puts it down. A new canvas tool joins the record rather than copying
  // the body a fifth time.
  const toggleTool = useMemo(() => {
    const toggle = (tool: RecordingCanvasTool) => () => {
      changeCanvasTool(canvasToolRef.current === tool ? null : tool);
    };
    return {
      arrow: toggle("arrow"),
      canvas: toggle("canvas"),
      crop: toggle("crop"),
      select: toggle("select"),
    };
  }, [changeCanvasTool]);
  // The crop tool is the one tool you are "in": Enter accepts what is framed
  // and Escape backs out of it, and both simply put the tool down - the crop
  // itself was committed as each handle was released. While it is in hand that
  // Escape outranks the timeline's own deselects.
  const isCropping = canvasTool === "crop";
  const leaveCropTool = useCallback(() => {
    if (canvasToolRef.current === "crop") changeCanvasTool(null);
  }, [changeCanvasTool]);
  return { changeCanvasTool, isCropping, leaveCropTool, toggleTool };
}
