// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode, type Ref } from "react";

import { cn } from "../../../lib/styling";

import type { SensitiveFind } from "../../../bindings/SensitiveFind";

/** Where a search found something in one turn's text, as `[from, to)`
 * letters, and whether it is the match gone to. */
export type FoundRange = { from: number; isCurrent: boolean; to: number };

/** The attribute that numbers a piece of a find that may be private, by its
 * place in the turn's finds, so a right click can tell which it fell on. */
export const SENSITIVE_ATTRIBUTE = "data-sensitive";

/** `text` with what a search found in it tinted as the Script transcript
 * tints it, the match gone to on the surface of a choice instead, and what
 * may be private underlined. A search and a private find can overlap, so
 * the text is cut wherever either starts or ends. `current` takes the
 * element of the match gone to, to bring it into view. */
export function markedPieces(
  text: string,
  {
    current,
    found,
    sensitive,
  }: {
    found: readonly FoundRange[];
    sensitive: readonly SensitiveFind[];
    current?: Ref<HTMLSpanElement>;
  },
): ReactNode {
  const cuts = new Set([0, text.length]);
  for (const { from, to } of found) cuts.add(from).add(to);
  for (const { end, start } of sensitive) cuts.add(start).add(end);
  const edges = [...cuts]
    .filter((at) => at >= 0 && at <= text.length)
    .sort((a, b) => a - b);
  const pieces: ReactNode[] = [];
  let wasCurrent = false;
  for (let index = 0; index + 1 < edges.length; index += 1) {
    const from = edges[index];
    const to = edges[index + 1];
    const search = found.find((range) => range.from <= from && range.to >= to);
    const private_ = sensitive.findIndex(
      (find) => find.start <= from && find.end >= to,
    );
    const words = text.slice(from, to);
    if (!search && private_ === -1) {
      pieces.push(words);
      continue;
    }
    const isCurrent = search?.isCurrent ?? false;
    pieces.push(
      <span
        className={cn(
          search && (isCurrent ? "bg-selected text-fg" : "found-text"),
          // A dotted border rather than a dotted underline: the browser
          // spaces an underline's dots from the line's start and drops the
          // last that does not fit, so it ended short of the words, where a
          // border's dots are spaced to span them exactly. Only its bottom
          // is drawn, which moves nothing along the line.
          private_ !== -1 && "border-b-2 border-dotted border-error",
        )}
        key={from}
        ref={isCurrent && !wasCurrent ? current : undefined}
        {...(private_ === -1 ? {} : { [SENSITIVE_ATTRIBUTE]: private_ })}
      >
        {words}
      </span>,
    );
    wasCurrent = isCurrent;
  }
  return pieces;
}

/** The find that may be private under `point` in `marks`, the marked text,
 * by its place in the turn's finds; null over none. */
export function sensitiveAt(
  marks: HTMLElement,
  { x, y }: { x: number; y: number },
) {
  for (const piece of marks.querySelectorAll(`[${SENSITIVE_ATTRIBUTE}]`)) {
    for (const rect of piece.getClientRects()) {
      if (
        x >= rect.left &&
        x <= rect.right &&
        y >= rect.top &&
        y <= rect.bottom
      )
        return Number(piece.getAttribute(SENSITIVE_ATTRIBUTE));
    }
  }
  return null;
}
