// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

/** The text column's width as a style: at most `width` points, centred, and
 * never wider than the frame; the whole frame without one. */
export const columnStyle = (width: number | null) => ({
  width: width === null ? "100%" : `min(${String(width)}px, 100%)`,
});
