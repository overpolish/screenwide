// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  animate,
  type MotionValue,
  type Transition,
  useMotionValue,
} from "motion/react";
import { useLayoutEffect, useRef } from "react";

import { useTravelRef } from "./island-follow";

// Where the island and what its row holds sit sideways as the row changes.
// Around a notch the island moves by the shift that keeps the notch over its
// gap, and the row is moved back by however far the island still has to go,
// so the side that did not change stays still while the black grows round
// the other. Without a notch the island grows evenly to both sides, and
// what it holds eases to its new centre with it rather than jumping there.

/** Writes the row's offset at once, so the frame the row is drawn in shows it
 * in its place. */
function placeRow(layer: HTMLElement | null, offset: number) {
  if (layer) layer.style.transform = `translateX(${String(offset)}px)`;
}

export function useRowOffset({
  fillsWidth,
  hasNotch,
  holdsContent,
  rowElement,
  rowTarget,
  shiftTarget,
  travel,
  width,
}: {
  /** The row spans the island, its parts at the black's ends, so there is
   * no centre for what it holds to ease to. */
  fillsWidth: boolean;
  hasNotch: boolean;
  /** Tucking away: what the island holds keeps its place while the island
   * moves, as it fades, rather than leaping to where the island is going. */
  holdsContent: boolean;
  rowElement: HTMLElement | null;
  /** Where the row should sit, in px from centre: under a notch, where its
   * gap is under the notch. The island sits there too unless it is wider
   * than the row and grows from one side of it (`shiftTarget`). */
  rowTarget: number;
  /** Where the island should sit, in px from centre. */
  shiftTarget: number;
  /** How the black moves to a new size (`useIslandFollow`). */
  travel: Transition;
  /** How wide the row's black should be, in px. */
  width: number;
}): {
  islandX: MotionValue<number>;
  /** Where the row sits so far, in px from centre: under a notch, where the
   * island has moved its gap to. */
  rowHeading: MotionValue<number>;
  rowLayerRef: React.RefObject<HTMLDivElement | null>;
  shapeWidth: MotionValue<number>;
} {
  const islandX = useMotionValue(shiftTarget);
  const shapeWidth = useMotionValue(width);
  const rowHeading = useMotionValue(rowTarget);
  const recentring = useMotionValue(0);
  const rowLayerRef = useRef<HTMLDivElement>(null);
  const rowWidthRef = useRef<number | null>(null);
  const travelRef = useTravelRef(travel);

  // The row's offset is written to a plain element, never handed to Motion:
  // Motion measures a part's layout without the transforms of the motion
  // elements above it, so a part that the offset holds still while the row
  // recentres round it would be seen to move, and glide back from where it
  // was, on top of the offset already keeping it there.
  useLayoutEffect(() => {
    const update = () => {
      placeRow(
        rowLayerRef.current,
        rowHeading.get() - islandX.get() + recentring.get(),
      );
    };
    const stops = [islandX, recentring, rowHeading].map((part) =>
      part.on("change", update),
    );
    return () => {
      stops.forEach((stop) => {
        stop();
      });
    };
  }, [islandX, recentring, rowHeading]);

  // The shift and the black's width start together on one curve: an edge
  // that should stay put is their sum, and started apart they would rock it.
  useLayoutEffect(() => {
    if (!holdsContent) rowHeading.set(rowTarget);
    const settling = animate(islandX, shiftTarget, travelRef.current);
    const widening = animate(shapeWidth, width, travelRef.current);
    placeRow(
      rowLayerRef.current,
      rowHeading.get() - islandX.get() + recentring.get(),
    );
    return () => {
      settling.stop();
      widening.stop();
    };
  }, [
    holdsContent,
    islandX,
    recentring,
    rowHeading,
    rowTarget,
    shapeWidth,
    shiftTarget,
    travelRef,
    width,
  ]);

  // After every render, before paint: the row's width only changes when what
  // it holds is rendered.
  useLayoutEffect(() => {
    const next = rowElement?.offsetWidth;
    const previous = rowWidthRef.current;
    rowWidthRef.current = next ?? null;
    if (next === undefined || previous === null || next === previous) return;
    if (hasNotch || fillsWidth) return;
    recentring.set(recentring.get() + (next - previous) / 2);
    animate(recentring, 0, travelRef.current);
    placeRow(
      rowLayerRef.current,
      rowHeading.get() - islandX.get() + recentring.get(),
    );
  });

  return { islandX, rowHeading, rowLayerRef, shapeWidth };
}
