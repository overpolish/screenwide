// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Span, type TimeMap } from "../timeline-scale";

// The order the stories' recording plays in, as the editor's timeline keeps
// it (`moveRecordingTimelineFootage`): the recording's stretches in the
// order they play. A move cuts the stretches where what moves starts and
// ends and where it lands, and the rest close up behind it. Whatever runs
// across one of those cuts, a beat, a lull, a scene, an annotation, a line,
// is split there into parts, so every part plays in one piece.

/** The recording's stretches, in recorded seconds, in the order they play.
 * Stretches that follow one another in both orders are one. */
export type PlayOrder = Span[];

/** `order` with a stretch ending and the next starting at `at`. */
function cutAt(order: PlayOrder, at: number): PlayOrder {
  return order.flatMap((piece) =>
    at > piece.start && at < piece.end
      ? [
          { end: at, start: piece.start },
          { end: piece.end, start: at },
        ]
      : [piece],
  );
}

/** `order` with stretches that run on in recorded time made one. */
function joined(order: PlayOrder): PlayOrder {
  const pieces: PlayOrder = [];
  for (const piece of order) {
    const last = pieces[pieces.length - 1] as Span | undefined;
    if (last && last.end === piece.start) last.end = piece.end;
    else pieces.push({ ...piece });
  }
  return pieces;
}

/** `order` with `span` played just before the moment `before`, or at the
 * end for null; a move onto the stretch itself changes nothing. */
export function moveStretch(
  order: PlayOrder,
  span: Span,
  before: number | null,
): PlayOrder {
  if (before !== null && before > span.start && before < span.end) return order;
  let pieces = cutAt(cutAt(order, span.start), span.end);
  if (before !== null) pieces = cutAt(pieces, before);
  const isMoved = (piece: Span) =>
    piece.start >= span.start && piece.end <= span.end;
  const rest = pieces.filter((piece) => !isMoved(piece));
  const at =
    before === null
      ? rest.length
      : rest.findIndex((piece) => piece.start === before);
  if (at < 0) return order;
  return joined([
    ...rest.slice(0, at),
    ...pieces.filter(isMoved),
    ...rest.slice(at),
  ]);
}

/** Where play jumps: the recorded moment each stretch after the first
 * starts at. */
export const jumpsOf = (order: PlayOrder) =>
  order.slice(1).map((piece) => piece.start);

/** `order` with `piece` played where it was recorded: after what it follows
 * in the recording, or before what follows it. */
function pieceBack(order: PlayOrder, piece: Span): PlayOrder {
  const rest = order.filter((each) => each !== piece);
  const after = rest.findIndex((each) => each.end === piece.start);
  const before = rest.findIndex((each) => each.start === piece.end);
  const index =
    after >= 0
      ? after + 1
      : before >= 0
        ? before
        : rest.filter((each) => each.start < piece.start).length;
  return joined([...rest.slice(0, index), piece, ...rest.slice(index)]);
}

/** `order` with the jump where the stretch starting at `at` plays put
 * right: one of the two stretches either side of it goes back where it was
 * recorded, whichever leaves the fewest jumps, so pressing any of the jumps
 * a move made undoes that move. */
export function putBack(order: PlayOrder, at: number): PlayOrder {
  const index = order.findIndex((each) => each.start === at);
  if (index < 1) return order;
  const [first, second] = [order[index - 1], order[index]]
    .map((piece) => ({
      length: piece.end - piece.start,
      order: pieceBack(order, piece),
    }))
    .sort(
      (one, other) =>
        one.order.length - other.order.length || one.length - other.length,
    );
  return (first.order.length <= second.order.length ? first : second).order;
}

/** Where `order`'s stretches start and end, where items are split. */
export const edgesOf = (order: PlayOrder) =>
  order.flatMap(({ end, start }) => [start, end]);

/** `items` with the parts of one item made one again where they meet at a
 * moment `order` no longer jumps at, as after a move is put back. Parts of
 * one item share the key `keyOf` reads; `join` makes one from two that
 * meet, the first before the second. */
export function rejoined<T>(
  items: T[],
  order: PlayOrder,
  {
    join,
    keyOf,
    spanOf,
  }: {
    join: (first: T, second: T) => T;
    keyOf: (item: T) => string;
    spanOf: (item: T) => Span;
  },
): T[] {
  const edges = new Set(order.flatMap(({ end, start }) => [start, end]));
  const sorted = [...items].sort(
    (first, second) => spanOf(first).start - spanOf(second).start,
  );
  const kept: T[] = [];
  for (const item of sorted) {
    const index = kept.findIndex(
      (each) =>
        keyOf(each) === keyOf(item) &&
        spanOf(each).end === spanOf(item).start &&
        !edges.has(spanOf(item).start),
    );
    if (index >= 0) kept[index] = join(kept[index], item);
    else kept.push(item);
  }
  return kept;
}

/** How the recording plays: its stretches in `order`, each lull or cut in
 * `paced` playing in its `plays` seconds, a cut in none. */
export function playedTime(
  order: PlayOrder,
  paced: (Span & { plays: number })[],
): TimeMap {
  let played = 0;
  const steps = cutAtAll(order, paced).map((step) => {
    const pace = paced.find(
      ({ end, start }) => step.start >= start && step.end <= end,
    );
    const rate = pace ? pace.plays / (pace.end - pace.start) : 1;
    const from = played;
    played += (step.end - step.start) * rate;
    return { ...step, from, rate };
  });
  return {
    duration: played,
    toPlayed: (recorded, isEnd = false) => {
      const step = steps.find(({ end, start }) =>
        isEnd
          ? recorded > start && recorded <= end
          : recorded >= start && recorded < end,
      );
      if (!step) return recorded <= 0 ? 0 : played;
      return step.from + (recorded - step.start) * step.rate;
    },
    toRecorded: (at) => {
      const step = steps.find(
        ({ end, from, rate, start }) =>
          rate > 0 && at >= from && at < from + (end - start) * rate,
      );
      if (!step) return steps.length > 0 ? steps[steps.length - 1].end : 0;
      return step.start + (at - step.from) / step.rate;
    },
  };
}

/** `order`'s stretches cut at every edge of `paced`, in play order. */
const cutAtAll = (order: PlayOrder, paced: Span[]) =>
  paced
    .flatMap(({ end, start }) => [start, end])
    .reduce((pieces, at) => cutAt(pieces, at), order);
