// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

/** The least room the text column leaves at its end for a cue to stand in,
 * in points: narrower, a picture there is too small to read at a glance. */
export const MARGIN_MIN = 160;

/** The room the text column leaves at each side of a frame `frameWidth`
 * points wide, in points: none where it fills the frame. */
export const columnMargin = (frameWidth: number, columnWidth: number | null) =>
  columnWidth === null ? 0 : Math.max(0, (frameWidth - columnWidth) / 2);
