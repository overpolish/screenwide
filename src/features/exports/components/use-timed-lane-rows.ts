// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import { RecordingTimelineEdit } from "../recording-timeline-edit";

import {
  layoutTimedLaneItems,
  stackTimedLaneFragments,
  TimedLaneItem,
} from "./timed-lane-layout";

/**
 * The single source of a lane's visible geometry: source-time items mapped
 * onto the output timeline, hidden fragments removed, and overlaps stacked
 * into sublanes. Every timed lane (shortcuts today; annotations and zooms
 * later) and any container sizing around one should derive from this, so a
 * lane and its chrome can never disagree about the row count.
 */
export function useTimedLaneRows<Item extends TimedLaneItem>({
  edit,
  hiddenFragmentIds,
  hiddenItemIds,
  items,
  sourceDurationMs,
}: {
  edit: RecordingTimelineEdit;
  items: Item[];
  sourceDurationMs: number;
  hiddenFragmentIds?: ReadonlySet<string>;
  hiddenItemIds?: ReadonlySet<Item["id"]>;
}) {
  return useMemo(
    () =>
      stackTimedLaneFragments(
        layoutTimedLaneItems({ edit, items, sourceDurationMs }).filter(
          (fragment) =>
            !hiddenFragmentIds?.has(fragment.fragmentId) &&
            !hiddenItemIds?.has(fragment.item.id),
        ),
      ),
    [edit, hiddenFragmentIds, hiddenItemIds, items, sourceDurationMs],
  );
}
