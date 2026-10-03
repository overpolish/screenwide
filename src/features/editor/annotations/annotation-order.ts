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
 * annotations bottom first. A redaction changes the picture itself, so it
 * goes under everything but the redactions, which a document keeps at its
 * bottom. A highlight recolours the page, so it goes under every spotlight
 * and every annotation drawn over the page; a spotlight goes over the
 * highlights and under the rest, so what is drawn next stays lit. Everything
 * else goes on top. The twin of `fresh_annotation_index` in
 * `src-tauri/src/editor/annotations/edit.rs`.
 */
export const freshAnnotationIndex = (
  kinds: readonly AnnotationKind[],
  kind: AnnotationKind,
) => {
  const beneath = (other: AnnotationKind) =>
    kind === "redact"
      ? other === "redact"
      : kind === "highlight"
        ? other === "highlight" || other === "redact"
        : kind === "spotlight"
          ? other === "highlight" || other === "spotlight" || other === "redact"
          : true;
  const index = kinds.findIndex((other) => !beneath(other));
  return index === -1 ? kinds.length : index;
};

/**
 * Whether annotations of kinds `a` and `b` stack against each other. A
 * redaction is applied to the picture before anything is drawn over it, so
 * redactions sit under every other annotation, and move only among
 * themselves.
 */
export const annotationKindsStack = (a: AnnotationKind, b: AnnotationKind) =>
  (a === "redact") === (b === "redact");

/** `items` with every redaction under every other annotation, each keeping
 * its own order: where annotations carried from elsewhere land. */
export const withRedactionsUnderneath = <Item>(
  items: Item[],
  kindOf: (item: Item) => AnnotationKind,
): Item[] => [
  ...items.filter((item) => kindOf(item) === "redact"),
  ...items.filter((item) => kindOf(item) !== "redact"),
];

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
 * over the next item it meets, backward under the previous one, or over the
 * last or under the first item it meets. The list is returned unchanged
 * where the move passes nothing it meets.
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
  if (arrangement === "front")
    rest.forEach((other, at) => {
      if (meets(item, other)) target = at + 1;
    });
  else if (arrangement === "back")
    target = rest.findIndex((other) => meets(item, other));
  else if (arrangement === "forward")
    target =
      rest.findIndex((other, at) => at >= index && meets(item, other)) + 1;
  else
    for (let at = index - 1; at >= 0; at -= 1)
      if (meets(item, rest[at])) {
        target = at;
        break;
      }
  return [...rest.slice(0, target), item, ...rest.slice(target)];
};

/**
 * The moves that change what the members of a group, the items `isMember`
 * picks, are drawn over: past an item outside the group that one of them
 * meets. A group of one offers what `availableArrangements` does.
 */
export const availableGroupArrangements = <Item>(
  items: Item[],
  isMember: (item: Item) => boolean,
  meets: (a: Item, b: Item) => boolean,
) => {
  let canBringForward = false;
  let canSendBackward = false;
  items.forEach((member, from) => {
    if (!isMember(member)) return;
    items.forEach((other, to) => {
      if (isMember(other) || !meets(member, other)) return;
      if (to > from) canBringForward = true;
      else canSendBackward = true;
    });
  });
  return { canBringForward, canSendBackward };
};

/**
 * `items` with the group's members moved through the drawing order together,
 * keeping their own stacking: a step forward or backward moves each member
 * past the next item it meets, the member furthest along the way moving first
 * so the ones behind it find the way clear, and a member held back by another
 * member stays where it is. The very top or bottom steps the members as far
 * as they go. The list is returned unchanged where the move passes nothing.
 */
export const arrangedGroup = <Item>(
  items: Item[],
  isMember: (item: Item) => boolean,
  {
    arrangement,
    meets,
  }: {
    arrangement: Arrangement;
    meets: (a: Item, b: Item) => boolean;
  },
): Item[] => {
  const { canBringForward, canSendBackward } = availableGroupArrangements(
    items,
    isMember,
    meets,
  );
  const forward = arrangement === "forward" || arrangement === "front";
  if (forward ? !canBringForward : !canSendBackward) return items;
  if (arrangement === "front" || arrangement === "back") {
    const step = arrangement === "front" ? "forward" : "backward";
    let moved = items;
    for (;;) {
      const next = arrangedGroup(moved, isMember, { arrangement: step, meets });
      if (next === moved) return moved;
      moved = next;
    }
  }
  const members = items.filter(isMember);
  if (forward) members.reverse();
  const step = forward ? 1 : -1;
  let stepped = items;
  for (const member of members) {
    const index = stepped.indexOf(member);
    let passed = index + step;
    while (
      passed >= 0 &&
      passed < stepped.length &&
      !meets(member, stepped[passed])
    )
      passed += step;
    if (passed < 0 || passed >= stepped.length || isMember(stepped[passed]))
      continue;
    stepped = arranged(stepped, index, { arrangement, meets });
  }
  return stepped;
};
