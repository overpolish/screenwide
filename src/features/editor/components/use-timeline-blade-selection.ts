// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useState } from "react";

import {
  RecordingTimelineEdit,
  recordingTimelineOutputToSource,
  recordingTimelineSourceToOutput,
} from "../recording-timeline-edit";

import { TimelineRangeSelection } from "./timeline-blade";

/**
 * Which blade tool is armed and what it has picked out. The blade and the
 * range tool are mutually exclusive, and arming either one drops whatever the
 * other had selected, so every transition lives here rather than spread
 * across the callers that trigger it.
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
  const [isActive, setIsActive] = useState(false);
  const [isRangeActive, setIsRangeActive] = useState(false);
  const [previewPosition, setPreviewPosition] = useState<number | null>(null);
  const [selectedSegmentId, setSelectedSegmentId] = useState<number | null>(
    null,
  );
  const [rangeSelection, setRangeSelection] =
    useState<TimelineRangeSelection | null>(null);
  const effectiveSelectedSegmentId =
    selectedSegmentId !== null &&
    edit.segments.some((segment) => segment.id === selectedSegmentId)
      ? selectedSegmentId
      : null;

  const setActive = useCallback((active: boolean) => {
    setIsActive(active);
    if (active) {
      setIsRangeActive(false);
      setRangeSelection(null);
      setSelectedSegmentId(null);
    } else setPreviewPosition(null);
  }, []);
  const toggle = useCallback(() => {
    setIsActive((active) => {
      if (active) setPreviewPosition(null);
      else {
        setIsRangeActive(false);
        setRangeSelection(null);
        setSelectedSegmentId(null);
      }
      return !active;
    });
  }, []);
  const changeRangeActive = useCallback((active: boolean) => {
    setIsRangeActive(active);
    if (active) {
      setIsActive(false);
      setPreviewPosition(null);
      setSelectedSegmentId(null);
    } else setRangeSelection(null);
  }, []);
  const toggleRange = useCallback(() => {
    setIsRangeActive((active) => {
      if (active) setRangeSelection(null);
      else {
        setIsActive(false);
        setPreviewPosition(null);
        setSelectedSegmentId(null);
      }
      return !active;
    });
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
    isActive,
    isRangeActive,
    previewAt,
    previewPosition,
    rangeSelection,
    selectSegment: setSelectedSegmentId,
    selectedSegmentId: effectiveSelectedSegmentId,
    setActive,
    setRangeActive: changeRangeActive,
    setRangeSelection: changeRangeSelection,
    toggle,
    toggleRange,
  };
}
