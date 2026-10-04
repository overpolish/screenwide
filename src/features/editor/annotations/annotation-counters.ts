// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { Annotation } from "./annotations";

/**
 * Every list with its counters numbered 1, 2, 3 as one run, in the order of
 * the numbers they already carry: on a tie, the earlier list first, then the
 * one earlier in its list.
 *
 * A counter keeps the number it was given: moving it, or changing which
 * annotations or layers it is drawn over, never renumbers it. Only the gaps
 * close, so deleting the second of three counters leaves 1 and 2 rather than
 * 1 and 3, and a delete, an undo and a reorder all land on the same numbers
 * without any of them knowing about counters. A list in which nothing moved
 * is handed back as it was, so an unchanged document is never rewritten.
 */
export const renumberedCounterLists = (
  lists: readonly Annotation[][],
): Annotation[][] => {
  const order = lists
    .flatMap((annotations, list) =>
      annotations.flatMap((annotation, index) =>
        annotation.shape.kind === "counter"
          ? [{ index, list, value: annotation.shape.value }]
          : [],
      ),
    )
    .sort((a, b) => a.value - b.value || a.list - b.list || a.index - b.index);
  const values = lists.map(() => new Map<number, number>());
  order.forEach(({ index, list }, place) => values[list].set(index, place + 1));
  return lists.map((annotations, list) => {
    const renumbered = annotations.map((annotation, index) => {
      const value = values[list].get(index);
      if (annotation.shape.kind !== "counter" || value === undefined)
        return annotation;
      return annotation.shape.value === value
        ? annotation
        : { ...annotation, shape: { ...annotation.shape, value } };
    });
    return renumbered.every(
      (annotation, index) => annotation === annotations[index],
    )
      ? annotations
      : renumbered;
  });
};
