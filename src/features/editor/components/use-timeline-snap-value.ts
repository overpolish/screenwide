// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useMemo, useRef } from "react";

import { RecordingAnnotationClip } from "../recording-annotations";
import { RecordingKeyboardTimelineItem } from "../types";

import { Playhead } from "./scrub-playhead";
import { TimelineBladeController } from "./timeline-blade";
import { TimelineSnap, timelineSnapTargets } from "./timeline-snap";

/**
 * The snap mode the lanes provide to their drags. Alt is read off the window
 * rather than the pointer: a native trim reports its travel through a
 * channel that carries no modifiers, and reading it one way for every drag
 * keeps the inversion the same wherever the pointer is.
 */
export function useTimelineSnapValue({
  annotationClips,
  blade,
  hiddenKeyboardFragmentIds,
  hiddenKeyboardItemIds,
  keyboardItems,
  playhead,
  sourceDurationMs,
}: {
  annotationClips: RecordingAnnotationClip[];
  blade: TimelineBladeController;
  hiddenKeyboardFragmentIds: ReadonlySet<string>;
  hiddenKeyboardItemIds: ReadonlySet<number>;
  keyboardItems: RecordingKeyboardTimelineItem[];
  playhead: Playhead;
  sourceDurationMs: number;
}): TimelineSnap {
  const altHeldRef = useRef(false);
  // A press on S mid-drag reaches the next move through the ref, not the
  // closure the drag took when it began.
  const activeRef = useRef(blade.isSnapActive);
  activeRef.current = blade.isSnapActive;
  useEffect(() => {
    const track = (event: KeyboardEvent) => {
      if (event.key === "Alt") altHeldRef.current = event.type === "keydown";
    };
    const release = () => {
      altHeldRef.current = false;
    };
    window.addEventListener("keydown", track, true);
    window.addEventListener("keyup", track, true);
    window.addEventListener("blur", release);
    return () => {
      window.removeEventListener("keydown", track, true);
      window.removeEventListener("keyup", track, true);
      window.removeEventListener("blur", release);
    };
  }, []);
  const { edit, setSnapGuidePosition } = blade;
  const gather = useCallback(
    (excludeAnnotationId?: string) =>
      timelineSnapTargets({
        annotationClips,
        edit,
        excludeAnnotationId,
        hiddenKeyboardFragmentIds,
        hiddenKeyboardItemIds,
        keyboardItems,
        playheadOutput: playhead.ratio(),
        sourceDurationMs,
      }),
    [
      annotationClips,
      edit,
      hiddenKeyboardFragmentIds,
      hiddenKeyboardItemIds,
      keyboardItems,
      playhead,
      sourceDurationMs,
    ],
  );
  return useMemo(
    () => ({
      gather,
      isActive: blade.isSnapActive,
      isSnapping: () => activeRef.current !== altHeldRef.current,
      showGuide: setSnapGuidePosition,
    }),
    [blade.isSnapActive, gather, setSnapGuidePosition],
  );
}
