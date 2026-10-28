// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ISLAND, type IslandAnchor, type IslandNotch } from "./island-geometry";

type Size = { height: number; width: number };

/** Where every part of the island goes, from what it holds: the row's and
 * section's measured sizes, where the section hangs, and the notch. Pure, so
 * the shape can be worked out the same way every render and animated to. */
export function islandLayout({
  anchor,
  hasSection,
  isShown,
  notch,
  reach = 0,
  rowSize,
  sectionSize,
}: {
  anchor: IslandAnchor;
  hasSection: boolean;
  isShown: boolean;
  notch: IslandNotch | null;
  rowSize: Size | null;
  sectionSize: Size | null;
  /** How much of the section's width is its aside (`IslandAside`), which
   * reaches past the row rather than widening it: at the start where the
   * section hangs from the end, at the end elsewhere. */
  reach?: number;
}) {
  const hidden = notch ?? ISLAND.speck;
  // Laid out at the row's measured size, which is kept while hidden, so a
  // section folds into and opens out of the same place.
  const rowOpen = rowSize ?? hidden;
  const isOpen = isShown && hasSection && sectionSize !== null;
  const held = sectionSize ?? { height: 0, width: 0 };
  // The row is fitted to the section without its aside, so the aside opens
  // past the row and nothing already there moves.
  const aside = Math.min(Math.max(reach, 0), held.width);
  const body = held.width - aside;
  // A section that would leave only a sliver of row beside it takes the
  // row's width instead, and one wider than the row widens the row with it.
  const spare = rowOpen.width - body;
  const isMerged =
    anchor === "center" ? spare / 2 < ISLAND.minStep : spare < ISLAND.minStep;
  const openWidth = isMerged ? Math.max(rowOpen.width, body) : rowOpen.width;
  const sectionWidth = (isMerged ? openWidth : body) + aside;
  const bodyOffset = isMerged
    ? 0
    : anchor === "start"
      ? 0
      : anchor === "end"
        ? spare
        : anchor === "center"
          ? spare / 2
          : underControl(anchor - body / 2, spare);
  const offset = anchor === "end" ? bodyOffset - aside : bodyOffset;
  // What the section holds rides at the box's start, and in a merged box
  // wider than itself sits where it is anchored, by an amount that moves on
  // the same curve: at the start or end, or centred. Placing it by layout
  // would shift it the moment a part of another width came in, before the
  // box had begun to change. Anchored to a side, it stays put as the row
  // grows from the other side round a notch.
  const slack = isMerged ? Math.max(0, sectionWidth - held.width) : 0;
  const inset =
    anchor === "start"
      ? 0
      : anchor === "end"
        ? slack
        : anchor === "center"
          ? slack / 2
          : Math.min(Math.max(anchor - held.width / 2, 0), slack);
  // The section is placed from the island's middle, which stays still as
  // the row grows out to both sides. Closed, it is a notch-wide box there;
  // it opens out of the middle to its place and folds back into it.
  const closed = hidden.width;
  const sectionStart = offset - openWidth / 2;
  const radius = isShown
    ? notch
      ? ISLAND.notchRadius
      : ISLAND.radius
    : notch
      ? ISLAND.hiddenNotchRadius
      : ISLAND.hiddenRadius;
  return {
    closed,
    /** Under a notch, how far the top flares out into the screen's edge. */
    flare: notch && isShown ? ISLAND.flare : 0,
    held,
    // While it opens or folds, what it holds is moved against the box so it
    // keeps its place and the black opens around it. Both move on one curve
    // from the same moment, so their sum stays put.
    inPlace: sectionStart + inset + closed / 2,
    inset,
    /** Hung from the row's end, so what it holds keeps to the box's end
     * while the box changes width (`island-section.tsx`). */
    isEndAnchored: anchor === "end",
    isOpen,
    radius,
    rowTarget: isShown ? rowOpen : hidden,
    sectionStart,
    sectionWidth,
    // Under the notch the island runs into the top of the screen.
    top: notch ? 0 : radius,
    width: !isShown
      ? hidden.width
      : isOpen && isMerged
        ? openWidth
        : rowOpen.width,
  };
}

/** Where a section centred under a control starts, `start` px in from the
 * row's start with `spare` px of row beside it: kept within the row, and
 * flush with an end it would leave only a sliver of row beside. */
function underControl(start: number, spare: number) {
  const within = Math.min(Math.max(start, 0), spare);
  if (within < ISLAND.minStep) return 0;
  if (spare - within < ISLAND.minStep) return spare;
  return within;
}
