// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export type AnnotationToolLayout<T> = {
  /** Reached through the overflow menu. Empty while every tool fits. */
  hidden: readonly T[];
  /** Shown after `visible`, standing in for whichever hidden tool was last
   * taken up. Null while every tool fits. */
  slot: T | null;
  /** Shown in toolbar order. */
  visible: readonly T[];
};

/**
 * Which annotation tools a strip with room for `capacity` buttons shows.
 *
 * Every tool shows while all of them fit. Otherwise the overflow button and
 * one slot take two of the places, the head of `order` takes the rest, and
 * the tail goes into the menu. The slot holds `slotted`, the last hidden tool
 * taken up, so picking from the menu swaps one button rather than shifting
 * the row; with none, or once `slotted` would be shown in its own place, the
 * slot holds the first tool that does not fit.
 */
export function annotationToolLayout<T>(
  order: readonly T[],
  capacity: number | null,
  slotted: T | null,
): AnnotationToolLayout<T> {
  if (capacity === null || capacity >= order.length)
    return { hidden: [], slot: null, visible: order };
  const count = Math.max(0, capacity - 2);
  const pinned =
    slotted !== null && order.indexOf(slotted) >= count ? slotted : null;
  if (pinned === null)
    return {
      hidden: order.slice(count + 1),
      slot: order[count] ?? null,
      visible: order.slice(0, count),
    };
  const rest = order.filter((item) => item !== pinned);
  return {
    hidden: rest.slice(count),
    slot: pinned,
    visible: rest.slice(0, count),
  };
}
