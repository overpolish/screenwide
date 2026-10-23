// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { animate, type MotionValue, useMotionValue } from "motion/react";
import { useLayoutEffect, useRef } from "react";

import { glide } from "./island-motion";

// Where the island and what its row holds sit sideways as the row changes.
// Around a notch the island moves by the shift that keeps the notch over its
// gap, and the row is moved back by however far the island still has to go,
// so the side that did not change stays still while the black grows round
// the other. Without a notch the island grows evenly to both sides, and
// what it holds eases to its new centre with it rather than jumping there.

/** Writes the row's offset at once. Motion draws a changed value on its next
 * frame, and the row is drawn now, so without this the first frame would
 * show what it holds jump before the offset caught up. */
function placeRow(layer: HTMLElement | null, offset: number) {
  if (layer) layer.style.transform = `translateX(${String(offset)}px)`;
}

export function useRowOffset({
  hasNotch,
  holdsContent,
  rowElement,
  rowTarget,
  shiftTarget,
  width,
}: {
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
  /** How wide the row's black should be, in px. */
  width: number;
}): {
  islandX: MotionValue<number>;
  rowLayerRef: React.RefObject<HTMLDivElement | null>;
  rowX: MotionValue<number>;
  /** Holds what hangs under the row still while the island moves sideways
   * under the notch. */
  sectionX: MotionValue<number>;
  shapeWidth: MotionValue<number>;
} {
  const islandX = useMotionValue(shiftTarget);
  const shapeWidth = useMotionValue(width);
  const heading = useMotionValue(shiftTarget);
  const rowHeading = useMotionValue(rowTarget);
  const recentring = useMotionValue(0);
  // A plain value kept current from its parts as each changes, not a derived
  // one: a derived value is worked out lazily, and Motion would draw its
  // stale result over the offset just written, for one frame.
  const rowX = useMotionValue(0);
  const sectionX = useMotionValue(0);
  const rowLayerRef = useRef<HTMLDivElement>(null);
  const rowWidthRef = useRef<number | null>(null);

  useLayoutEffect(() => {
    const update = () => {
      sectionX.set(heading.get() - islandX.get());
      rowX.set(rowHeading.get() - islandX.get() + recentring.get());
    };
    const stops = [heading, islandX, recentring, rowHeading].map((part) =>
      part.on("change", update),
    );
    return () => {
      stops.forEach((stop) => {
        stop();
      });
    };
  }, [heading, islandX, recentring, rowHeading, rowX, sectionX]);

  // The shift and the black's width start together on one curve: an edge
  // that should stay put is their sum, and started apart they would rock it.
  useLayoutEffect(() => {
    if (!holdsContent) {
      heading.set(shiftTarget);
      rowHeading.set(rowTarget);
    }
    const settling = animate(islandX, shiftTarget, glide);
    const widening = animate(shapeWidth, width, glide);
    placeRow(
      rowLayerRef.current,
      rowHeading.get() - islandX.get() + recentring.get(),
    );
    return () => {
      settling.stop();
      widening.stop();
    };
  }, [
    heading,
    holdsContent,
    islandX,
    recentring,
    rowHeading,
    rowTarget,
    shapeWidth,
    shiftTarget,
    width,
  ]);

  // After every render, before paint: the row's width only changes when what
  // it holds is rendered.
  useLayoutEffect(() => {
    const next = rowElement?.offsetWidth;
    const previous = rowWidthRef.current;
    rowWidthRef.current = next ?? null;
    if (next === undefined || previous === null || next === previous) return;
    if (hasNotch) return;
    recentring.set(recentring.get() + (next - previous) / 2);
    animate(recentring, 0, glide);
    placeRow(
      rowLayerRef.current,
      rowHeading.get() - islandX.get() + recentring.get(),
    );
  });

  return { islandX, rowLayerRef, rowX, sectionX, shapeWidth };
}
