// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  RecordingTimelineEdit,
  recordingTimelineRetainedDuration,
} from "../recording-timeline-edit";

export type TimedLaneItem = {
  endMs: number;
  id: number | string;
  startMs: number;
};

export type TimedLaneFragment<Item extends TimedLaneItem> = {
  fragmentId: string;
  item: Item;
  outputEnd: number;
  outputStart: number;
  segmentId: number;
};

const clamp = (value: number) => Math.max(0, Math.min(1, value));

/**
 * Lays source-time items onto the magnetic output timeline. A ranged item that
 * crosses a segment boundary becomes multiple independently addressable visual
 * fragments, which is also suitable for future captions and annotations.
 */
export function layoutTimedLaneItems<Item extends TimedLaneItem>({
  edit,
  items,
  sourceDurationMs,
}: {
  edit: RecordingTimelineEdit;
  items: Item[];
  sourceDurationMs: number;
}): TimedLaneFragment<Item>[] {
  if (sourceDurationMs <= 0) return [];
  const retainedDuration = recordingTimelineRetainedDuration(edit);
  if (retainedDuration <= 0) return [];

  return items.flatMap((item) => {
    const sourceStart = clamp(item.startMs / sourceDurationMs);
    const sourceEnd = clamp(
      Math.max(item.startMs, item.endMs) / sourceDurationMs,
    );
    let retainedBefore = 0;
    const fragments: TimedLaneFragment<Item>[] = [];

    for (const [index, segment] of edit.segments.entries()) {
      const playbackRate = segment.playbackRate ?? 1;
      const segmentDuration =
        (segment.sourceEnd - segment.sourceStart) / playbackRate;
      const intersectionStart = Math.max(sourceStart, segment.sourceStart);
      const intersectionEnd = Math.min(sourceEnd, segment.sourceEnd);
      const isPoint = sourceStart === sourceEnd;
      const containsPoint =
        sourceStart >= segment.sourceStart &&
        (sourceStart < segment.sourceEnd ||
          (index === edit.segments.length - 1 &&
            sourceStart === segment.sourceEnd));
      if (intersectionStart < intersectionEnd || (isPoint && containsPoint)) {
        const outputStart =
          (retainedBefore +
            (intersectionStart - segment.sourceStart) / playbackRate) /
          retainedDuration;
        const outputEnd =
          (retainedBefore +
            (intersectionEnd - segment.sourceStart) / playbackRate) /
          retainedDuration;
        fragments.push({
          fragmentId: `${String(item.id)}:${segment.id.toString()}`,
          item,
          outputEnd,
          outputStart,
          segmentId: segment.id,
        });
      }
      retainedBefore += segmentDuration;
    }
    return fragments;
  });
}

export type StackedLaneFragment<Item extends TimedLaneItem> =
  TimedLaneFragment<Item> & {
    row: number;
    /**
     * The same item continuing seamlessly from the previous fragment on this
     * row - a segment split, not a separate occurrence - so the lane renders
     * the pair joined and carries the label only once.
     */
    continuesPrevious: boolean;
    continuedByNext: boolean;
    /**
     * Carries the item's label: the widest fragment of its seam run, so a
     * sliver at a segment boundary never swallows the whole run's label.
     */
    showLabel: boolean;
  };

const SEAM_EPSILON = 1e-9;

/**
 * Assigns overlapping fragments to stacked sublanes so simultaneous items
 * stay individually visible. Fragments that never coincide share row zero,
 * keeping the lane a single row tall in the common case.
 */
export function stackTimedLaneFragments<Item extends TimedLaneItem>(
  fragments: TimedLaneFragment<Item>[],
): { fragments: StackedLaneFragment<Item>[]; rowCount: number } {
  const ordered = [...fragments].sort(
    (left, right) =>
      left.outputStart - right.outputStart || left.outputEnd - right.outputEnd,
  );
  const rowEnds: number[] = [];
  const rowLast: (StackedLaneFragment<Item> | undefined)[] = [];
  const runs: StackedLaneFragment<Item>[][] = [];
  const rowRun: number[] = [];
  const stacked = ordered.map((fragment) => {
    let row = rowEnds.findIndex((end) => fragment.outputStart >= end);
    if (row === -1) {
      row = rowEnds.length;
      rowEnds.push(fragment.outputEnd);
    } else {
      rowEnds[row] = fragment.outputEnd;
    }
    const previous = rowLast[row];
    const continuesPrevious =
      previous !== undefined &&
      previous.item.id === fragment.item.id &&
      Math.abs(previous.outputEnd - fragment.outputStart) < SEAM_EPSILON;
    const placed = {
      ...fragment,
      continuedByNext: false,
      continuesPrevious,
      row,
      showLabel: false,
    };
    if (continuesPrevious && previous) {
      previous.continuedByNext = true;
      runs[rowRun[row] ?? -1]?.push(placed);
    } else {
      rowRun[row] = runs.length;
      runs.push([placed]);
    }
    rowLast[row] = placed;
    return placed;
  });
  for (const run of runs) {
    let widest: StackedLaneFragment<Item> | undefined;
    for (const member of run) {
      if (
        !widest ||
        member.outputEnd - member.outputStart >
          widest.outputEnd - widest.outputStart
      ) {
        widest = member;
      }
    }
    if (widest) widest.showLabel = true;
  }
  return { fragments: stacked, rowCount: Math.max(rowEnds.length, 1) };
}
