// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { CameraBackgroundRange } from "../../recording/camera/camera-background-ranges";

import { CameraBackgroundLane } from "./camera-background-strip";
import { TimedSpan } from "./timed-span-strip";
import { useTimedSpanSelection } from "./use-timed-span-selection";

/**
 * The camera's background ranges for the timeline stories, held in the
 * story's own state. `pending` is still to be worked out wherever a range
 * covers it, as it would be while the matte is being made.
 */
export function useStoryCameraBackground({
  pending = [],
  ranges: initialRanges = [],
}: {
  pending?: readonly TimedSpan[];
  ranges?: CameraBackgroundRange[];
}): CameraBackgroundLane {
  const [ranges, setRanges] = useState(initialRanges);
  // The story's playhead only moves the drawn playhead, so the choice stays
  // where it is made rather than following it.
  const selection = useTimedSpanSelection({
    clips: ranges,
    commit: setRanges,
    currentId: null,
  });
  return {
    onChange: setRanges,
    onClearSelection: selection.onClear,
    onSelect: selection.onSelect,
    onSelectSwept: selection.onSelectSwept,
    pendingSpans: pending.length > 0 ? () => pending : undefined,
    ranges,
    selectedIds: selection.ids,
  };
}
