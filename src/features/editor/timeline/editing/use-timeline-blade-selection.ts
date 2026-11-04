// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useState } from "react";

import {
  RecordingTimelineEdit,
  recordingTimelineOutputToSource,
  recordingTimelineSourceToOutput,
} from "./recording-timeline-edit";
import { TimelineRangeSelection } from "./timeline-blade";

/**
 * What the timeline's edits have picked out: a stretch of the edited video,
 * a segment, where a split would land, and whether drags snap. The strip's
 * tools in hand are its own (`use-strip-tools.ts`).
 */
export function useTimelineBladeSelection({
  edit,
  snap,
  snapOutput,
}: {
  edit: RecordingTimelineEdit;
  snap: (sourcePosition: number) => number;
  snapOutput: (outputPosition: number) => number;
}) {
  const [previewPosition, setPreviewPosition] = useState<number | null>(null);
  const [selectedSegmentId, setSelectedSegmentId] = useState<number | null>(
    null,
  );
  const [rangeSelection, setRangeSelection] =
    useState<TimelineRangeSelection | null>(null);
  // On by default: lining an annotation up with its neighbour is the usual
  // reason to drag it, and Alt pulls an edge off a target for the odd time.
  const [isSnapActive, setIsSnapActive] = useState(true);
  const [snapGuidePosition, setSnapGuidePosition] = useState<number | null>(
    null,
  );
  const effectiveSelectedSegmentId =
    selectedSegmentId !== null &&
    edit.segments.some((segment) => segment.id === selectedSegmentId)
      ? selectedSegmentId
      : null;

  const toggleSnap = useCallback(() => {
    setIsSnapActive((active) => !active);
  }, []);
  const changeRangeSelection = useCallback(
    (anchor: number, focus: number) => {
      const start = snapOutput(Math.min(anchor, focus));
      const end = snapOutput(Math.max(anchor, focus));
      setRangeSelection(start === end ? null : { end, start });
    },
    [snapOutput],
  );
  const clearRangeSelection = useCallback(() => {
    setRangeSelection(null);
  }, []);
  const previewAt = useCallback(
    (outputPosition: number) => {
      // A join already carries a cut, so there is nothing to promise there:
      // the line hides at the one position the blade cannot act on and
      // follows the pointer everywhere else, edges included.
      const snappedSource = snap(
        recordingTimelineOutputToSource(edit, outputPosition),
      );
      const cuttable = edit.segments.some(
        (segment) =>
          snappedSource > segment.sourceStart &&
          snappedSource < segment.sourceEnd,
      );
      setPreviewPosition(
        cuttable ? recordingTimelineSourceToOutput(edit, snappedSource) : null,
      );
    },
    [edit, snap],
  );
  const clearPreview = useCallback(() => {
    setPreviewPosition(null);
  }, []);
  const clearSelection = useCallback(() => {
    setSelectedSegmentId(null);
  }, []);

  return {
    clearPreview,
    clearRangeSelection,
    clearSelection,
    isSnapActive,
    previewAt,
    previewPosition,
    rangeSelection,
    selectSegment: setSelectedSegmentId,
    selectedSegmentId: effectiveSelectedSegmentId,
    setRangeSelection: changeRangeSelection,
    setSnapActive: setIsSnapActive,
    setSnapGuidePosition,
    snapGuidePosition,
    toggleSnap,
  };
}
