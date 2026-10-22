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
  rowSize,
  sectionSize,
}: {
  anchor: IslandAnchor;
  hasSection: boolean;
  isShown: boolean;
  notch: IslandNotch | null;
  rowSize: Size | null;
  sectionSize: Size | null;
}) {
  const hidden = notch ?? ISLAND.speck;
  // Laid out at the row's measured size, which is kept while hidden, so a
  // section folds into and opens out of the same place.
  const rowOpen = rowSize ?? hidden;
  const isOpen = isShown && hasSection && sectionSize !== null;
  const held = sectionSize ?? { height: 0, width: 0 };
  // A section that would leave only a sliver of row beside it takes the
  // row's width instead, and one wider than the row widens the row with it.
  const spare = rowOpen.width - held.width;
  const isMerged =
    anchor === "center" ? spare / 2 < ISLAND.minStep : spare < ISLAND.minStep;
  const openWidth = isMerged
    ? Math.max(rowOpen.width, held.width)
    : rowOpen.width;
  const sectionWidth = isMerged ? openWidth : held.width;
  const offset = isMerged
    ? 0
    : anchor === "start"
      ? 0
      : anchor === "end"
        ? spare
        : spare / 2;
  // What the section holds rides at the box's start, and is centred only in
  // a merged box wider than itself, by an amount that moves on the same
  // curve. Centring it by layout would shift it the moment a part of another
  // width came in, before the box had begun to change.
  const inset = isMerged ? Math.max(0, (sectionWidth - held.width) / 2) : 0;
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
    flare: notch ? (isShown ? ISLAND.flare : ISLAND.hiddenFlare) : 0,
    held,
    // While it opens or folds, what it holds is moved against the box so it
    // keeps its place and the black opens around it. Both move on one curve
    // from the same moment, so their sum stays put.
    inPlace: sectionStart + inset + closed / 2,
    inset,
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
