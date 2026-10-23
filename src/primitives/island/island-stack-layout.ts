// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ISLAND, type IslandAnchor } from "./island-geometry";
import { islandLayout } from "./island-layout";

type Layout = ReturnType<typeof islandLayout>;
type Size = { height: number; width: number };

/** The island laid out, with a second line under its row where it has one,
 * and where it sits sideways: under a notch, where the row's gap is under
 * the notch, or where the stack of row and line puts it. */
export function layOutIsland({
  below,
  direction,
  rowShift,
  ...parts
}: Parameters<typeof islandLayout>[0] & {
  /** The line under the row, measured; none without one. */
  below: Size | null;
  direction: "ltr" | "rtl";
  rowShift: number;
}): Layout & { islandShift: number } {
  const layout = islandLayout(parts);
  return below
    ? stackUnderRow({
        anchor: parts.anchor,
        below,
        direction,
        layout,
        rowShift,
      })
    : { ...layout, islandShift: rowShift };
}

/**
 * The island laid out with a second line under its row, in the row's black,
 * and the section hanging under that line rather than under the row. The
 * row keeps its gap under the notch; the line is centred under the notch
 * whatever the row does beside it; the black of the two is one, from the
 * further-out start to the further-out end, so a row growing on one side
 * widens the island on that side alone and nothing under it moves. The
 * section hangs from the line's start or end, as it would from a row of its
 * own; edges within a step of each other meet, so no sliver of a step is
 * left between them (`ISLAND.minStep`).
 *
 * Worked out on the screen, from the notch's middle, or the island's where
 * there is none, then given back in the island's own terms: its sideways
 * shift, its width, the black's height, and where the section and what it
 * holds start along the line of the text.
 */
function stackUnderRow({
  anchor,
  below,
  direction,
  layout,
  rowShift,
}: {
  anchor: IslandAnchor;
  below: Size;
  direction: "ltr" | "rtl";
  layout: Layout;
  /** Where the row's middle sits from the notch's, in px across the screen,
   * so that its gap is under the notch. */
  rowShift: number;
}): Layout & { islandShift: number } {
  const isRtl = direction === "rtl";
  const row = layout.rowTarget;
  const rowLeft = rowShift - row.width / 2;
  const rowRight = rowShift + row.width / 2;
  const left = Math.min(rowLeft, -below.width / 2);
  const right = Math.max(rowRight, below.width / 2);
  const middle = (left + right) / 2;
  const stacked = {
    ...layout,
    islandShift: middle,
    rowTarget: { height: row.height + below.height, width: right - left },
    width: right - left,
  };
  if (!layout.isOpen) return stacked;

  // The line's start and end on the screen: in a right-to-left layout its
  // start is the right-hand edge.
  const held = layout.held.width;
  const lineStart = isRtl ? below.width / 2 : -below.width / 2;
  const physicalAnchor =
    anchor === "center"
      ? "center"
      : (anchor === "start") !== isRtl
        ? "left"
        : "right";
  const heldLeft =
    physicalAnchor === "left"
      ? Math.min(lineStart, -lineStart)
      : physicalAnchor === "right"
        ? Math.max(lineStart, -lineStart) - held
        : -held / 2;
  const heldRight = heldLeft + held;
  const boxLeft = heldLeft - left < ISLAND.minStep ? left : heldLeft;
  const boxRight = right - heldRight < ISLAND.minStep ? right : heldRight;
  // Along the line of the text, from the island's middle and from the box's
  // start.
  const sectionStart = isRtl ? middle - boxRight : boxLeft - middle;
  const inset = isRtl ? boxRight - heldRight : heldLeft - boxLeft;
  return {
    ...stacked,
    inPlace: sectionStart + inset + layout.closed / 2,
    inset,
    sectionStart,
    sectionWidth: boxRight - boxLeft,
  };
}
