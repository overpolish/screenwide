// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { animate, type MotionValue, useTransform } from "motion/react";
import { useLayoutEffect, useRef, useState } from "react";

import { px } from "./island-geometry";
import { glide } from "./island-motion";

import type { islandLayout } from "./island-layout";
import type { IslandShapeValues } from "./island-outline-sync";

/** Where a section's box goes, and how it gets there: its start and width
 * are moved here, toward where it hangs, the middle as the island comes
 * out or goes in, or under the row as it folds in place; or, while one
 * that spans the row opens in place, along with the row's own width. They
 * are driven from here rather than by the box's own animation, which would
 * set them back where they began the moment the row took them over. */
export function useSectionPlace({
  isFromMiddle,
  isOpening,
  layout,
  rowWidth,
  stretch,
  values,
}: {
  isFromMiddle: boolean;
  isOpening: boolean;
  layout: ReturnType<typeof islandLayout>;
  /** The row's width as it is drawn, moving as it grows or shrinks. */
  rowWidth: MotionValue<number>;
  /** How far what the section holds is stretched across its box. */
  stretch: number;
  values: IslandShapeValues["section"];
}) {
  const { closed, inset, isOpen, rowTarget, sectionStart, sectionWidth } =
    layout;
  // A section opens out of the middle, and folds back into it, only as the
  // island comes out of the notch or goes back in. While the island is out,
  // a section opens where it hangs, growing down from under what opened it,
  // and folds up there again; its last place is kept for that, and how far
  // it was stretched, so it folds from where it was shown rather than
  // jumping to its own width and place the moment it starts.
  const [lastPlace, setLastPlace] = useState({
    inset,
    start: sectionStart,
    stretch,
    width: sectionWidth,
  });
  if (
    isOpen &&
    (lastPlace.inset !== inset ||
      lastPlace.start !== sectionStart ||
      lastPlace.stretch !== stretch ||
      lastPlace.width !== sectionWidth)
  ) {
    setLastPlace({ inset, start: sectionStart, stretch, width: sectionWidth });
  }
  const heldStretch = isOpen ? stretch : lastPlace.stretch;
  // Folding in place, the box narrows to fit under the row as it folds, so
  // the black closes up as one: a section wider than the row would
  // otherwise fold at its own width while the row has already drawn in
  // beside it. One that fits under the row folds where it hangs. What it
  // holds is held where it was against the box's move, so the black closes
  // over it.
  const foldWidth = Math.min(lastPlace.width, rowTarget.width);
  const foldStart = Math.min(
    Math.max(lastPlace.start, -rowTarget.width / 2),
    rowTarget.width / 2 - foldWidth,
  );
  // Opening in place, the box widens out from where it last folded, under
  // the row, on the row's own curve, as it folds back in: what it holds is
  // put straight where it will rest and held there against the box's move,
  // so the black opens round it while its height grows. Where the box was
  // as it started is kept for the whole opening, so the move has one start.
  // A section never open before has no such place (its box sits closed in
  // the island's middle), so it starts as wide as the row is, under it; set
  // here, before the box's own animation reads where it starts from, and
  // the same however often it is set.
  const isOpeningInPlace = isOpening && !isFromMiddle;
  const [hasBeenOpen, setHasBeenOpen] = useState(false);
  const [openFrom, setOpenFrom] = useState<number | null>(null);
  if (isOpeningInPlace && openFrom === null) {
    if (!hasBeenOpen) {
      const row = rowWidth.get();
      values.width.set(row);
      values.start.set(px(-row / 2));
    }
    setOpenFrom(Number.parseFloat(values.start.get()) || 0);
  }
  if (!isOpeningInPlace && openFrom !== null) setOpenFrom(null);
  if (isOpen && !hasBeenOpen) setHasBeenOpen(true);
  // One that spans the row as it opens rides on the row's own width rather
  // than running a move of its own, so the two are one shape every frame:
  // a part coming into the row with it, as a control that comes with the
  // section, would otherwise set the row moving a frame before the section
  // is measured and open, and the row would flare out past it. What it
  // holds is held against the box from the same value.
  const followsRow =
    isOpeningInPlace && Math.abs(sectionWidth - layout.width) < 0.5;
  const targetStart = isOpen
    ? sectionStart
    : isFromMiddle
      ? -closed / 2
      : foldStart;
  const targetWidth = isOpen ? sectionWidth : isFromMiddle ? closed : foldWidth;
  // The first place is taken at once, as a box shown already in place.
  const isPlacedRef = useRef(false);
  useLayoutEffect(() => {
    if (followsRow) {
      const follow = (width: number) => {
        values.width.set(width);
        values.start.set(px(-width / 2));
      };
      follow(rowWidth.get());
      return rowWidth.on("change", follow);
    }
    if (!isPlacedRef.current) {
      isPlacedRef.current = true;
      values.start.set(px(targetStart));
      values.width.set(targetWidth);
      return;
    }
    const moves = [
      animate(values.start, px(targetStart), glide),
      animate(values.width, targetWidth, glide),
    ];
    return () => {
      moves.forEach((move) => {
        move.stop();
      });
    };
  }, [followsRow, rowWidth, targetStart, targetWidth, values]);
  // While the box follows the row open, or folds in place, what it holds is
  // held where it will rest by how far the box is from there, read off the
  // box's own start every frame, so the two can never drift: nothing once
  // the box arrives. Elsewhere nothing is held here.
  const holdAt = followsRow
    ? sectionStart
    : !isOpen && !isFromMiddle
      ? foldStart
      : null;
  const holdOffset = useTransform(values.start, (start) =>
    holdAt === null ? "0px" : px(holdAt - (Number.parseFloat(start) || 0)),
  );
  return {
    foldStart,
    followsRow,
    heldStretch,
    holdOffset,
    isHolding: holdAt !== null,
    isOpeningInPlace,
    lastPlace,
    openFrom,
  };
}
