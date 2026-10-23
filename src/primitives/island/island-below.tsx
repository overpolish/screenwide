// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AnimatePresence,
  animate,
  motion,
  type MotionValue,
  useMotionValue,
  useTransform,
} from "motion/react";
import { type ReactNode, useLayoutEffect, useRef } from "react";

import { ISLAND } from "./island-geometry";
import { focusIn, glide } from "./island-motion";

// A second line under the island's row, in the row's black: centred under
// the notch, or the island's middle where there is none, whatever the row
// does beside it, so the island moving sideways for the row never carries
// it. When its own width changes it eases to its new centre rather than
// jumping there, as the row does without a notch. The section hangs under
// it (`island-stack-layout.ts`).

export function IslandBelow({
  children,
  isShown,
  islandX,
  onElement,
  top,
  width,
}: {
  children: ReactNode;
  isShown: boolean;
  /** The island's sideways place, which the line is held against. */
  islandX: MotionValue<number>;
  /** The element holding the line, to be measured. */
  onElement: (element: HTMLDivElement | null) => void;
  /** Where the line starts down the black: under the row. */
  top: number;
  /** The line's measured width, eased to its new centre as it changes. */
  width: number | undefined;
}) {
  const recentring = useMotionValue(0);
  const x = useTransform(() => recentring.get() - islandX.get());
  const widthRef = useRef(width);
  useLayoutEffect(() => {
    const previous = widthRef.current;
    widthRef.current = width;
    if (previous === undefined || width === undefined || previous === width)
      return;
    recentring.set(recentring.get() + (width - previous) / 2);
    const easing = animate(recentring, 0, glide);
    return () => {
      easing.stop();
    };
  }, [recentring, width]);

  return (
    <motion.div
      className="absolute inset-x-0 flex justify-center"
      style={{ top, x }}
    >
      <AnimatePresence initial={false}>
        {isShown ? (
          <motion.div
            key="below"
            {...focusIn}
            className="flex w-max flex-col gap-inline"
            initial={false}
            ref={onElement}
            style={{
              paddingBottom: ISLAND.rowInset,
              paddingInline: ISLAND.rowInset,
            }}
          >
            {children}
          </motion.div>
        ) : null}
      </AnimatePresence>
    </motion.div>
  );
}
