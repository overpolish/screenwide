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
  TimedLaneFragment<Item> & { row: number };

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
  const stacked = ordered.map((fragment) => {
    let row = rowEnds.findIndex((end) => fragment.outputStart >= end);
    if (row === -1) {
      row = rowEnds.length;
      rowEnds.push(fragment.outputEnd);
    } else {
      rowEnds[row] = fragment.outputEnd;
    }
    return { ...fragment, row };
  });
  return { fragments: stacked, rowCount: Math.max(rowEnds.length, 1) };
}
