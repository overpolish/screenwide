// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { animate, type MotionValue, useMotionValue } from "motion/react";
import { useLayoutEffect, useRef } from "react";

import type { Curve } from "./island-motion";

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
  curve,
  hasNotch,
  rowElement,
  shiftTarget,
  width,
}: {
  curve: Curve;
  hasNotch: boolean;
  rowElement: HTMLElement | null;
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
      rowX.set(sectionX.get() + recentring.get());
    };
    const stops = [heading, islandX, recentring].map((part) =>
      part.on("change", update),
    );
    return () => {
      stops.forEach((stop) => {
        stop();
      });
    };
  }, [heading, islandX, recentring, rowX, sectionX]);

  // The shift and the black's width start together on one curve: an edge
  // that should stay put is their sum, and started apart they would rock it.
  useLayoutEffect(() => {
    heading.set(shiftTarget);
    const settling = animate(islandX, shiftTarget, curve);
    const widening = animate(shapeWidth, width, curve);
    placeRow(
      rowLayerRef.current,
      heading.get() - islandX.get() + recentring.get(),
    );
    return () => {
      settling.stop();
      widening.stop();
    };
  }, [curve, heading, islandX, recentring, shapeWidth, shiftTarget, width]);

  // After every render, before paint: the row's width only changes when what
  // it holds is rendered.
  useLayoutEffect(() => {
    const next = rowElement?.offsetWidth;
    const previous = rowWidthRef.current;
    rowWidthRef.current = next ?? null;
    if (next === undefined || previous === null || next === previous) return;
    if (hasNotch) return;
    recentring.set(recentring.get() + (next - previous) / 2);
    animate(recentring, 0, curve);
    placeRow(
      rowLayerRef.current,
      heading.get() - islandX.get() + recentring.get(),
    );
  });

  return { islandX, rowLayerRef, rowX, sectionX, shapeWidth };
}
