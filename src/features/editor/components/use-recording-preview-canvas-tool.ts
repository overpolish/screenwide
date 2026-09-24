// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useCallback, useEffect, useMemo, useRef } from "react";

import { drawingToolKind } from "../tool-panels/tool-registry";
import { RecordingTrackId, RecordingVideoTrackId } from "../types";

import { RecordingCanvasTool } from "./recording-crop-toggle";
import { useRecordingPreviewSelection } from "./use-recording-preview-selection";
import { toolDisagreesWithAnnotation } from "./use-tool-follows-annotation";

import type { AnnotationKind } from "../../../components/shared/annotation-style/types";

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
  selectedAnnotationKind,
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
  /** The shape of the annotation in hand, so a tool that draws the other one
   * knows to let it go. */
  selectedAnnotationKind: AnnotationKind | null;
  setCanvasTool: (tool: RecordingCanvasTool) => void;
  onSelectedTrackChange?: (trackId: RecordingTrackId | null) => void;
}) {
  const canvasToolRef = useRef(canvasTool);
  canvasToolRef.current = canvasTool;
  const selectedKindRef = useRef(selectedAnnotationKind);
  selectedKindRef.current = selectedAnnotationKind;
  const changeCanvasTool = useCallback(
    (next: RecordingCanvasTool) => {
      if (drawingToolKind(next) !== null) {
        if (toolDisagreesWithAnnotation(next, selectedKindRef.current))
          clearAnnotationRef.current();
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
      counter: toggle("counter"),
      crop: toggle("crop"),
      select: toggle("select"),
      text: toggle("text"),
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
