// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode, type Ref } from "react";

import { cn } from "../../../lib/styling";

/** Where a search found something in one turn's text, as `[from, to)`
 * letters, and whether it is the match gone to. */
export type FoundRange = { from: number; isCurrent: boolean; to: number };

/** `text` with what a search found in it tinted as the Script transcript
 * tints it, the match gone to on the surface of a choice instead. `current`
 * takes the element of the match gone to, to bring it into view. */
export function foundPieces(
  text: string,
  ranges: readonly FoundRange[],
  current?: Ref<HTMLSpanElement>,
): ReactNode {
  const pieces: ReactNode[] = [];
  let at = 0;
  for (const { from, isCurrent, to } of ranges) {
    if (from > at) pieces.push(text.slice(at, from));
    pieces.push(
      <span
        className={cn(isCurrent ? "bg-selected text-fg" : "found-text")}
        key={from}
        ref={isCurrent ? current : undefined}
      >
        {text.slice(from, to)}
      </span>,
    );
    at = to;
  }
  if (at < text.length) pieces.push(text.slice(at));
  return pieces;
}
