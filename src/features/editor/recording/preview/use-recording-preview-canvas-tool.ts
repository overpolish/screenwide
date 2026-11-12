// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useCallback, useEffect, useMemo, useRef } from "react";

import { useSelectToolMemory } from "../../annotations/use-select-tool-memory";
import { toolDisagreesWithAnnotation } from "../../choice/editor-choice";
import { drawingToolKind } from "../../tools/tool-registry";
import { RecordingTrackId, RecordingVideoTrackId } from "../../types";

import { RecordingCanvasTool } from "./recording-canvas-tool";

import type { AnnotationKind } from "../../../annotations/types";

/** Picking a canvas tool up and putting it down, and what each change clears
 * on its way in. */
export function useRecordingPreviewCanvasTool({
  activeVideoTrack,
  annotations,
  bakeCamera,
  canvasTool,
  clearAnnotationRef,
  clearOverlaySelection,
  hasVisiblePanes,
  onSelectedTrackChange,
  setCanvasTool,
}: {
  activeVideoTrack: RecordingVideoTrackId | null;
  /** The annotations in hand. `selectedKind` is their shape, so a tool that
   * draws the other one knows to let them go. */
  annotations: {
    onSelectedIdsChange: (ids: readonly string[]) => void;
    selectedIds: ReadonlySet<string>;
    selectedKind: AnnotationKind | null;
  };
  bakeCamera: boolean;
  canvasTool: RecordingCanvasTool;
  clearAnnotationRef: RefObject<() => void>;
  /** Lets go of the shortcuts and captions chosen, which a drawing tool
   * taken up does. */
  clearOverlaySelection: () => void;
  hasVisiblePanes: boolean;
  setCanvasTool: (tool: RecordingCanvasTool) => void;
  onSelectedTrackChange?: (trackId: RecordingTrackId | null) => void;
}) {
  const canvasToolRef = useRef(canvasTool);
  canvasToolRef.current = canvasTool;
  const selectedKindRef = useRef(annotations.selectedKind);
  selectedKindRef.current = annotations.selectedKind;
  const rememberSelection = useSelectToolMemory(
    annotations.selectedIds,
    annotations.onSelectedIdsChange,
  );
  const changeCanvasTool = useCallback(
    (next: RecordingCanvasTool) => {
      // A mode (crop, frame, scene) lets the annotations in hand go, and
      // putting it away does not bring them back.
      const isMode = drawingToolKind(next) === null && next !== "select";
      if (drawingToolKind(next) !== null) {
        if (toolDisagreesWithAnnotation(next, selectedKindRef.current))
          clearAnnotationRef.current();
        clearOverlaySelection();
        onSelectedTrackChange?.(
          bakeCamera ? "primary" : (activeVideoTrack ?? "primary"),
        );
      } else if (next !== "select") clearAnnotationRef.current();
      setCanvasTool(next);
      rememberSelection(canvasToolRef.current, next, isMode);
    },
    [
      setCanvasTool,
      clearOverlaySelection,
      onSelectedTrackChange,
      bakeCamera,
      activeVideoTrack,
      clearAnnotationRef,
      rememberSelection,
    ],
  );
  // A canvas tool acts on the panes on screen. When there are none - the
  // last video track switched off, or the picture not laid out yet as the
  // editor opens - there is nothing for it to act on, so it is put down and
  // its panel goes with it rather than staying up over the ribbon saying
  // nothing is selected. Put down, Select is in hand, as everywhere else:
  // the native side reads no tool at all as a picture that picks nothing.
  useEffect(() => {
    if (!hasVisiblePanes && canvasToolRef.current !== "select")
      setCanvasTool("select");
  }, [hasVisiblePanes, setCanvasTool]);
  // One toggle apiece, built from one function: picking up the tool already in
  // hand puts it down, which leaves Select in hand. A new canvas tool joins
  // the record rather than copying the body a fifth time.
  const toggleTool = useMemo(() => {
    const toggle = (tool: RecordingCanvasTool) => () => {
      changeCanvasTool(canvasToolRef.current === tool ? "select" : tool);
    };
    return {
      arrow: toggle("arrow"),
      canvas: toggle("canvas"),
      counter: toggle("counter"),
      crop: toggle("crop"),
      draw: toggle("draw"),
      focus: toggle("focus"),
      highlight: toggle("highlight"),
      image: toggle("image"),
      magnify: toggle("magnify"),
      redact: toggle("redact"),
      scene: toggle("scene"),
      select: toggle("select"),
      shape: toggle("shape"),
      spotlight: toggle("spotlight"),
      text: toggle("text"),
    };
  }, [changeCanvasTool]);
  // The crop tool is the one tool you are "in": Enter accepts what is framed
  // and Escape backs out of it - the crop itself was committed as each handle
  // was released. While it is in hand that Escape outranks the timeline's own
  // deselects.
  const isCropping = canvasTool === "crop";
  return { changeCanvasTool, isCropping, toggleTool };
}
