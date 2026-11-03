// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject } from "react";

import { DropBoundary, dropCaret } from "./transcript-drop-geometry";

/**
 * The caret that marks where dragged words would land, drawn over the
 * transcript between the words so it moves without laying the text out
 * again. It is measured from the words' own boxes, which are physical, so
 * it is placed from the physical left in any reading direction.
 */
export function TranscriptDropCaret({
  boundary,
  rootRef,
  spansRef,
}: {
  boundary: DropBoundary;
  rootRef: RefObject<HTMLElement | null>;
  spansRef: RefObject<HTMLElement[]>;
}) {
  const root = rootRef.current;
  if (!root) return null;
  const spans = spansRef.current;
  const caret = dropCaret(
    spans.map((element) => element.getBoundingClientRect()),
    boundary,
    (index) => getComputedStyle(spans[index]).direction === "rtl",
  );
  if (!caret) return null;
  const origin = root.getBoundingClientRect();
  return (
    <span
      aria-hidden
      className="pointer-events-none absolute top-0 w-0.5 rounded-full bg-primary ltr:left-0 rtl:left-0"
      style={{
        height: caret.bottom - caret.top,
        // Centred on the boundary: the caret is two pixels wide.
        translate: `${String(caret.x - origin.left - 1)}px ${String(caret.top - origin.top)}px`,
      }}
    />
  );
}
