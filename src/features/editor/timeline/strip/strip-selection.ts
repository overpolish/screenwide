// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Span } from "../timeline-scale";

/** A beat or lull, named by its kind and id. */
export type StripItem = { id: string; kind: "beat" | "lull" };

/** What is chosen on the strip: one of its parts, several beats and lulls
 * gathered with Shift, or a stretch the scissors drew, in recorded seconds.
 */
export type StripSelection =
  | { id: string; kind: "annotation" | "beat" | "lull" | "scene" }
  | { items: StripItem[]; kind: "items" }
  | (Span & { kind: "range" });

type Part = Exclude<StripSelection["kind"], "items" | "range">;

/** The stretch a selection covers, found among the parts that have one;
 * several items cover from the first one's start to the last one's end. */
export function selectionSpan(
  selection: StripSelection | null,
  parts: Record<Part, { id: string; span: Span }[]>,
): Span | null {
  if (!selection) return null;
  if (selection.kind === "range") return selection;
  const spanOf = (kind: Part, id: string) =>
    parts[kind].find((part) => part.id === id)?.span;
  if (selection.kind !== "items")
    return spanOf(selection.kind, selection.id) ?? null;
  const spans = selection.items.flatMap(
    ({ id, kind }) => spanOf(kind, id) ?? [],
  );
  if (spans.length === 0) return null;
  return {
    end: Math.max(...spans.map(({ end }) => end)),
    start: Math.min(...spans.map(({ start }) => start)),
  };
}

/** The beats and lulls `selection` chooses. */
export const chosenItems = (selection: StripSelection | null): StripItem[] =>
  selection?.kind === "items"
    ? selection.items
    : selection?.kind === "beat" || selection?.kind === "lull"
      ? [{ id: selection.id, kind: selection.kind }]
      : [];

/** Whether `selection` chooses the part `id` of `kind`. */
export const isChosen = (
  selection: StripSelection | null,
  kind: Part,
  id: string,
) =>
  (selection?.kind === kind && selection.id === id) ||
  (selection?.kind === "items" &&
    selection.items.some((item) => item.kind === kind && item.id === id));

/** `selection` with `item` taken in, or left out if it was in, as Shift
 * pressing it does: one item left is chosen alone, none chooses nothing.
 * Anything else chosen gives way to the item. */
export function toggled(
  selection: StripSelection | null,
  item: StripItem,
): StripSelection | null {
  const items = chosenItems(selection);
  if (items.length === 0) return item;
  const isIn = items.some(
    (each) => each.kind === item.kind && each.id === item.id,
  );
  const next = isIn
    ? items.filter((each) => each.kind !== item.kind || each.id !== item.id)
    : [...items, item];
  if (next.length === 0) return null;
  return next.length === 1 ? next[0] : { items: next, kind: "items" };
}
