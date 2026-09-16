// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { TIMED_LANE_ROW_HEIGHT_PX } from "./timed-lane-layout";

/**
 * The timeline band is laid out entirely from spacing tokens, but the band's
 * height is a number in JS: the divider drags it, and the minimum stop is an
 * arithmetic sum of the rows that must stay visible.
 * These mirror the `@theme` tokens in `src/index.css` so the arithmetic below
 * is written in tokens rather than in magic numbers.
 */

/**
 * `--spacing-control`, the gap between rows in the band. Not to be merged with
 * `TIMED_LANE_ROW_INSET_PX` in `timed-lane-layout.ts`: that one is the inset
 * inside a single lane row, and the two carrying the same number today is a
 * coincidence rather than one idea written twice.
 */
export const CONTROL_GAP_PX = 4;
/** `--spacing-control-inset`, the band's own padding unit. */
export const CONTROL_INSET_PX = 8;
/**
 * `--spacing-control-height`, the height of one control row - the same token a
 * lane row is laid out to, so the lane constant stands for both.
 */
const CONTROL_HEIGHT_PX = TIMED_LANE_ROW_HEIGHT_PX;

/**
 * The playback transport row, which stands above the scroller: it is part of
 * the band's height without being part of its scrolled content.
 * Only a fallback - the real row is measured, since a control can grow it.
 */
export const TIMELINE_PLAYBACK_ROW_HEIGHT_PX =
  CONTROL_HEIGHT_PX + 2 * CONTROL_INSET_PX;

/**
 * The ruler/zoom-toolbar row plus the gap to the first lane under it. The row
 * is laid out to `h-control-height`, a token rather than a measurement, so the
 * band can count it without observing it. Like the transport row above, it
 * stands outside the scroller: only the lane rows scroll under it.
 */
export const TIMELINE_HEADER_BLOCK_HEIGHT_PX =
  CONTROL_HEIGHT_PX + CONTROL_GAP_PX;

/**
 * The height a lanes block wants: the fixed ruler row standing above the
 * scroller plus the lane rows that scroll under it, measured. This is the
 * `contentHeight` the two stops below are given for a timeline.
 */
export const timelineLanesHeight = (rowsHeight: number) =>
  TIMELINE_HEADER_BLOCK_HEIGHT_PX + rowsHeight;

/** How many lane rows the band refuses to shrink below. */
const MINIMUM_VISIBLE_LANE_ROWS = 2;

/**
 * The smallest useful band: the playback transport row and the ruler/toolbar
 * row with its gap, both standing above the scroller, then inside it two lane
 * rows with the gap between them and the lane strip's bottom inset. The resize
 * handle is drawn over the band's edge rather than above it, so it takes no
 * height here. Nothing rounded - it is the honest sum.
 */
export const TIMELINE_MIN_HEIGHT_PX =
  TIMELINE_PLAYBACK_ROW_HEIGHT_PX +
  TIMELINE_HEADER_BLOCK_HEIGHT_PX +
  MINIMUM_VISIBLE_LANE_ROWS * TIMED_LANE_ROW_HEIGHT_PX +
  (MINIMUM_VISIBLE_LANE_ROWS - 1) * CONTROL_GAP_PX +
  CONTROL_INSET_PX;

/** The band's height is only ever set through this stop pair. */
export const clampTimelineHeight = (value: number, maximum: number) =>
  Math.max(TIMELINE_MIN_HEIGHT_PX, Math.min(maximum, value));

/**
 * The height that shows the whole timeline block: the transport row plus the
 * block under it - for a timeline, the ruler row and the lane rows that scroll
 * beneath it - clamped like a drag.
 */
export const fitTimelineHeight = ({
  contentHeight,
  headerHeight,
  maximum,
}: {
  contentHeight: number;
  headerHeight: number;
  maximum: number;
}) => clampTimelineHeight(headerHeight + contentHeight, maximum);

/**
 * How tall the meter beside the lanes stands. It does not scroll with them, so
 * it is sized to the strip of rows on screen - stopping where those rows' own
 * bottom inset begins rather than running to the band's edge. The viewport is
 * measured, so it already accounts for every lane and sublane whatever the
 * platform's control height is; no lane-height cap is needed here, because the
 * band's own maximum never lets the viewport grow past the rows it holds.
 */
export const timelineMeterHeight = (visibleHeight: number) =>
  Math.max(TIMED_LANE_ROW_HEIGHT_PX, visibleHeight - CONTROL_INSET_PX);

/**
 * The largest useful band: never taller than the transport row plus the
 * timeline block under it (ruler row included, since it no longer scrolls), so
 * dragging the divider up stops at the last lane instead of opening a strip of
 * empty band, and never taller than the space the preview can spare. An unmeasured block has nothing to cap against and
 * yields to the space.
 */
export const timelineMaximumHeight = ({
  contentHeight,
  headerHeight,
  spaceHeight,
}: {
  contentHeight: number;
  headerHeight: number;
  spaceHeight: number;
}) => {
  const space = Math.max(TIMELINE_MIN_HEIGHT_PX, spaceHeight);
  if (contentHeight <= 0) return space;
  return Math.max(
    TIMELINE_MIN_HEIGHT_PX,
    Math.min(space, headerHeight + contentHeight),
  );
};
