// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Span } from "../timeline-scale";

/** A beat or lull, named by its kind and id. */
export type StripItem = { id: string; kind: "beat" | "lull" };

/** What is chosen on the strip: one of its parts, several beats and lulls
 * gathered with Shift, shortcuts, one or gathered, or a stretch the
 * scissors drew, in recorded seconds. */
export type StripSelection =
  | { id: string; kind: "annotation" | "beat" | "lull" | "scene" }
  | { ids: string[]; kind: "shortcuts" }
  | { items: StripItem[]; kind: "items" }
  | (Span & { kind: "range" });

type Part = Exclude<StripSelection["kind"], "items" | "range" | "shortcuts">;

/** A stretch, and where choosing it brings the playhead in recorded seconds
 * where that is not its start (a scene once it has arrived). */
type Landing = Span & { landsAt?: number };

/** The stretch a selection covers, found among the parts that have one;
 * several items cover from the first one's start to the last one's end. */
export function selectionSpan(
  selection: StripSelection | null,
  parts: Record<Part, { id: string; span: Landing }[]>,
): Landing | null {
  if (!selection) return null;
  if (selection.kind === "range") return selection;
  // Pressed where they are, shortcuts are in view already.
  if (selection.kind === "shortcuts") return null;
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

/** How far before a part's start, in seconds, the playhead still stands
 * at it: a seek lands on the frame nearest, which may fall just short. */
const SAME_MOMENT_S = 1e-3;

/** Whether the playhead, at `at` in recorded seconds, stands outside
 * `span`, so choosing the part brings it to the part's start. */
export const isPlayheadOutside = (at: number, span: Span) =>
  at < span.start - SAME_MOMENT_S || at >= span.end;

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
