// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The order annotations are drawn in: a document keeps them bottom first, and
 * the compositor draws them in that order, so the list is the stacking.
 */

import { AnnotationShape } from "./annotations";

type AnnotationKind = AnnotationShape["kind"];

/**
 * Where a fresh annotation of `kind` goes among `kinds`, the document's
 * annotations bottom first. A highlight recolours the page, so it goes under
 * every spotlight and every annotation drawn over the page; a spotlight goes
 * over the highlights and under the rest, so what is drawn next stays lit.
 * Everything else goes on top. A redaction is applied to the picture wherever
 * it sits, so it never holds a fresh one back. The twin of
 * `fresh_annotation_index` in `src-tauri/src/editor/annotations/edit.rs`.
 */
export const freshAnnotationIndex = (
  kinds: readonly AnnotationKind[],
  kind: AnnotationKind,
) => {
  const beneath = (other: AnnotationKind) =>
    kind === "highlight"
      ? other === "highlight" || other === "redact"
      : kind === "spotlight"
        ? other === "highlight" || other === "spotlight" || other === "redact"
        : true;
  const index = kinds.findIndex((other) => !beneath(other));
  return index === -1 ? kinds.length : index;
};

/** A move through the drawing order. */
export type Arrangement = "back" | "backward" | "forward" | "front";

/**
 * The moves that change what the item at `index` is drawn over. `meets` says
 * whether two items are ever drawn together, so passing an item that never
 * shares a frame with this one is not a move at all.
 */
export const availableArrangements = <Item>(
  items: Item[],
  index: number,
  meets: (a: Item, b: Item) => boolean,
) => ({
  canBringForward: items.some(
    (other, at) => at > index && meets(items[index], other),
  ),
  canSendBackward: items.some(
    (other, at) => at < index && meets(items[index], other),
  ),
});

/**
 * `items` with the one at `index` moved through the drawing order: forward
 * over the next item it meets, backward under the previous one, or to the
 * very top or bottom. The list is returned unchanged where the move passes
 * nothing it meets.
 */
export const arranged = <Item>(
  items: Item[],
  index: number,
  {
    arrangement,
    meets,
  }: {
    arrangement: Arrangement;
    meets: (a: Item, b: Item) => boolean;
  },
): Item[] => {
  const item = items[index];
  const { canBringForward, canSendBackward } = availableArrangements(
    items,
    index,
    meets,
  );
  const forward = arrangement === "forward" || arrangement === "front";
  if (forward ? !canBringForward : !canSendBackward) return items;
  const rest = items.filter((_, at) => at !== index);
  let target = 0;
  if (arrangement === "front") target = rest.length;
  else if (arrangement === "forward")
    target =
      rest.findIndex((other, at) => at >= index && meets(item, other)) + 1;
  else if (arrangement === "backward")
    for (let at = index - 1; at >= 0; at -= 1)
      if (meets(item, rest[at])) {
        target = at;
        break;
      }
  return [...rest.slice(0, target), item, ...rest.slice(target)];
};
