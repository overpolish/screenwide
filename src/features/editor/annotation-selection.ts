// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { selectTimelineItem } from "./components/timeline-item-selection";

const NO_ANNOTATIONS: ReadonlySet<string> = new Set();

/**
 * The chosen ids that still name an annotation. A group of two or more holds
 * only annotations that `groupable` allows: a pinned recording annotation
 * follows its content on its own, so it is never carried with others, and
 * one that gains a pin while grouped drops out. The same set comes back when
 * nothing was let go, so an unchanged choice keeps its identity.
 */
export const chosenAnnotationIds = (
  annotationIds: readonly string[],
  chosen: ReadonlySet<string>,
  groupable: (id: string) => boolean,
): ReadonlySet<string> => {
  if (chosen.size === 0) return chosen;
  const present = new Set(annotationIds);
  let kept = [...chosen].filter((id) => present.has(id));
  if (kept.length > 1) kept = kept.filter(groupable);
  if (kept.length === chosen.size) return chosen;
  return kept.length === 0 ? NO_ANNOTATIONS : new Set(kept);
};

/**
 * The choice after a click on `id`: that annotation alone, or with `toggle`
 * held, the choice with it added or taken away. A toggle cannot bring an
 * annotation that may not be grouped into a choice that already holds others.
 */
export const toggledAnnotationIds = (
  current: ReadonlySet<string>,
  id: string,
  {
    groupable,
    toggle,
  }: { groupable: (id: string) => boolean; toggle: boolean },
): ReadonlySet<string> =>
  toggle && current.size > 0 && !current.has(id) && !groupable(id)
    ? current
    : selectTimelineItem(current, id, toggle);

/** The choice after a band swept over `ids`: those alone, or added to what
 * was chosen already. */
export const sweptAnnotationIds = (
  current: ReadonlySet<string>,
  ids: readonly string[],
  additive: boolean,
): ReadonlySet<string> =>
  additive ? new Set([...current, ...ids]) : new Set(ids);

/**
 * What a menu opened on `id` acts on: everything chosen when it is one of
 * several chosen together, and otherwise it alone.
 */
export const annotationMenuTargets = (
  id: string,
  chosen: ReadonlySet<string>,
): ReadonlySet<string> =>
  chosen.size > 1 && chosen.has(id) ? chosen : new Set([id]);

/**
 * What a delete acts on: the annotation the halo is showing when it is not
 * one of the chosen, and otherwise everything chosen.
 *
 * The halo is under the pointer, so it is what the hand is pointing at; the
 * choice is what it last pointed at. Pointing at one of the chosen means the
 * whole choice. An id that names no annotation on this layer - a stale hover
 * from a layer that has moved on - is no target at all.
 */
export const annotationDeleteTargets = (
  annotationIds: readonly string[],
  hoveredId: string | null,
  chosen: ReadonlySet<string>,
): ReadonlySet<string> => {
  const hovered =
    hoveredId !== null && annotationIds.includes(hoveredId) ? hoveredId : null;
  if (hovered !== null && !chosen.has(hovered)) return new Set([hovered]);
  return chosenAnnotationIds(annotationIds, chosen, () => true);
};
