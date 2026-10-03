// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Annotation } from "./annotations";

/**
 * The spotlights shown together lay one shade, and the picture under it is
 * blurred when any of them blurs, so they carry one Blur setting between
 * them. Each of `groups` is a set of spotlights shown together. A group takes
 * the setting one of its spotlights was just switched to; failing that, the
 * one the spotlights it already held carry, which a spotlight that has just
 * joined it adopts. `previous` is each spotlight as it stood before the edit,
 * by id, holding only the ones that were in place: a spotlight missing from
 * it has just joined. Answers the spotlights whose setting changes, by id.
 */
export const sharedSpotlightBlurs = (
  groups: readonly (readonly Annotation[])[],
  previous: ReadonlyMap<string, Annotation>,
): Map<string, boolean> => {
  const changes = new Map<string, boolean>();
  for (const group of groups) {
    const switched = group.find((spotlight) => {
      const before = previous.get(spotlight.id);
      return before !== undefined && before.style.blur !== spotlight.style.blur;
    });
    const settled = group.find((spotlight) => previous.has(spotlight.id));
    const blur = (switched ?? settled)?.style.blur;
    if (blur === undefined) continue;
    for (const spotlight of group)
      if (spotlight.style.blur !== blur) changes.set(spotlight.id, blur);
  }
  return changes;
};

/** `annotation` with the Blur setting `changes` gives it, if any. */
export const withSharedBlur = (
  annotation: Annotation,
  changes: ReadonlyMap<string, boolean>,
): Annotation => {
  const blur = changes.get(annotation.id);
  return blur === undefined
    ? annotation
    : { ...annotation, style: { ...annotation.style, blur } };
};
