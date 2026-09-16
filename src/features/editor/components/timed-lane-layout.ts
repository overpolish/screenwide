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
    continuedByNext: boolean;
    /**
     * The same item continuing seamlessly from the previous fragment on this
     * row - a segment split, not a separate occurrence - so the lane renders
     * the pair joined and carries the label only once.
     */
    continuesPrevious: boolean;
    row: number;
    /**
     * Carries the item's label: the widest fragment of its seam run, so a
     * sliver at a segment boundary never swallows the whole run's label.
     */
    showLabel: boolean;
  };

/**
 * One sublane row's height in CSS pixels. Mirrors the `--spacing-control-height`
 * token, which every row in the timeline band is laid out to - 1.5rem (24px) on
 * macOS, 2rem (32px) in the Windows skin - read the same way `main.tsx` stamps
 * `data-platform`. The lane's absolutely positioned fragments need the number
 * in JS, so it is kept here beside the stacking that produces rows.
 */
export const TIMED_LANE_ROW_HEIGHT_PX =
  typeof navigator !== "undefined" && /Windows/i.test(navigator.userAgent)
    ? 32
    : 24;

/**
 * The breathing space a fragment leaves above and below itself inside its row,
 * so stacked sublanes read as separate bars rather than one block. Mirrors
 * `--spacing-control`. It is the inset within a row, NOT the gap between the
 * band's rows - `CONTROL_GAP_PX` in `timeline-band-metrics.ts` is that one, and
 * the two carrying the same number today is a coincidence, not a shared idea.
 */
const TIMED_LANE_ROW_INSET_PX = 4;

/**
 * Where one fragment sits in its lane, in CSS pixels: every timed lane places
 * its bars absolutely against the same row grid, so they all measure it here
 * rather than each repeating the arithmetic. Shortcuts and annotations use it
 * today; the captions and zoom lanes planned next are its other callers.
 */
export const timedLaneFragmentBox = (row: number) => ({
  height: TIMED_LANE_ROW_HEIGHT_PX - 2 * TIMED_LANE_ROW_INSET_PX,
  top: row * TIMED_LANE_ROW_HEIGHT_PX + TIMED_LANE_ROW_INSET_PX,
});

const SEAM_EPSILON = 1e-9;

/**
 * How many sublane rows the minimum width alone may grow a lane to. Genuinely
 * simultaneous items always earn a row of their own; this bounds only the
 * extra rows that widening a brief item to a readable size asks for, so a
 * dense run of keystrokes seen zoomed out cannot grow the lane without limit.
 */
const MINIMUM_WIDTH_ROW_LIMIT = 4;

/**
 * Converts a lane's minimum item width into the output-ratio span a fragment
 * of that width covers. Lane fragments are positioned against the zoomed
 * viewport content, which is the lane's own width times the zoom, so the span
 * a fixed pixel width claims shrinks as the timeline is zoomed in.
 */
export const timedLaneMinimumSpan = ({
  contentWidthPx,
  minimumItemWidthPx,
  zoom,
}: {
  contentWidthPx: number;
  minimumItemWidthPx: number;
  zoom: number;
}) => {
  const zoomedWidth = contentWidthPx * zoom;
  if (zoomedWidth <= 0 || minimumItemWidthPx <= 0) return 0;
  return Math.min(1, minimumItemWidthPx / zoomedWidth);
};

/**
 * Assigns overlapping fragments to stacked sublanes so simultaneous items
 * stay individually visible. Fragments that never coincide share row zero,
 * keeping the lane a single row tall in the common case.
 *
 * `minimumSpan` is the width the lane will not paint a fragment narrower than,
 * as a share of the output timeline. Rows are claimed by what a fragment
 * occupies on screen rather than by its timing alone, so two brief items too
 * close together to be drawn side by side stack instead of overprinting.
 */
export function stackTimedLaneFragments<Item extends TimedLaneItem>(
  fragments: TimedLaneFragment<Item>[],
  { minimumSpan = 0 }: { minimumSpan?: number } = {},
): { fragments: StackedLaneFragment<Item>[]; rowCount: number } {
  const ordered = [...fragments].sort(
    (left, right) =>
      left.outputStart - right.outputStart || left.outputEnd - right.outputEnd,
  );
  // Seam-run members waive the minimum width, exactly as the lane paints
  // them: an inflated sliver would cover the fragment it continues into. An
  // item yields at most one fragment per segment, so more than one fragment
  // of the same item is a run.
  const seen = new Set<Item["id"]>();
  const runItemIds = new Set<Item["id"]>();
  for (const fragment of ordered) {
    if (seen.has(fragment.item.id)) {
      runItemIds.add(fragment.item.id);
    } else {
      seen.add(fragment.item.id);
    }
  }
  /** What the fragment covers once drawn, which is what a row is claimed by. */
  const painted = (fragment: TimedLaneFragment<Item>) => {
    if (minimumSpan <= 0 || runItemIds.has(fragment.item.id)) {
      return { end: fragment.outputEnd, start: fragment.outputStart };
    }
    // The lane will not paint a fragment narrower than `minimumSpan`, so the
    // row it claims extends to what is drawn, not to where its timing ends.
    // (A fragment at the far right already clips against the lane's overflow;
    // that pre-existing edge needs no help here.)
    return {
      end: Math.max(fragment.outputEnd, fragment.outputStart + minimumSpan),
      start: fragment.outputStart,
    };
  };
  const rowEnds: number[] = [];
  const rowLast: (StackedLaneFragment<Item> | undefined)[] = [];
  const runs: StackedLaneFragment<Item>[][] = [];
  const rowRun: number[] = [];
  const stacked = ordered.map((fragment) => {
    const box = painted(fragment);
    let row = rowEnds.findIndex((end) => box.start >= end);
    if (row === -1 && rowEnds.length >= MINIMUM_WIDTH_ROW_LIMIT) {
      // Past that budget only a real overlap in time earns another row; the
      // rest share a row and are drawn overlapping, as they were before the
      // minimum width was accounted for.
      row = rowEnds.findIndex((end) => fragment.outputStart >= end);
    }
    if (row === -1) {
      row = rowEnds.length;
      rowEnds.push(box.end);
    } else {
      rowEnds[row] = Math.max(rowEnds[row], box.end);
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
    if (continuesPrevious) {
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
