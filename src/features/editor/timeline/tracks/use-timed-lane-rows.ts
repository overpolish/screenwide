// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import { RecordingTimelineEdit } from "../editing/recording-timeline-edit";

import {
  LaneFragment,
  layoutTimedLaneItems,
  layoutTimelineLaneItems,
  stackTimedLaneFragments,
  TimedLaneItem,
} from "./timed-lane-layout";

/**
 * The single source of a lane's visible geometry: items mapped onto the
 * output timeline, hidden fragments removed, and overlaps stacked into
 * sublanes. Items are timed in the source, as shortcuts are, unless
 * `outputLengthMs` says they are timed on the edited video that long, as
 * captions are. Every timed lane and any container sizing around one should
 * derive from this, so a lane and its chrome can never disagree about the
 * row count.
 */
export function useTimedLaneRows<Item extends TimedLaneItem>({
  edit,
  hiddenFragmentIds,
  hiddenItemIds,
  items,
  minimumSpan = 0,
  outputLengthMs,
  sourceDurationMs,
}: {
  edit: RecordingTimelineEdit;
  items: Item[];
  sourceDurationMs: number;
  hiddenFragmentIds?: ReadonlySet<string>;
  hiddenItemIds?: ReadonlySet<Item["id"]>;
  /** The on-screen floor each fragment is painted at, as an output-ratio span. */
  minimumSpan?: number;
  outputLengthMs?: number;
}) {
  return useMemo(() => {
    const laid: LaneFragment<Item>[] =
      outputLengthMs === undefined
        ? layoutTimedLaneItems({ edit, items, sourceDurationMs })
        : layoutTimelineLaneItems(items, outputLengthMs);
    return stackTimedLaneFragments<Item, LaneFragment<Item>>(
      laid.filter(
        (fragment) =>
          !hiddenFragmentIds?.has(fragment.fragmentId) &&
          !hiddenItemIds?.has(fragment.item.id),
      ),
      { minimumSpan },
    );
  }, [
    edit,
    hiddenFragmentIds,
    hiddenItemIds,
    items,
    minimumSpan,
    outputLengthMs,
    sourceDurationMs,
  ]);
}
