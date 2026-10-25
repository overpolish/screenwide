// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnimatePresence, motion, useAnimate } from "motion/react";
import { type ReactNode, useEffect, useRef } from "react";

import { ISLAND } from "../../../primitives/island/island-geometry";

import { breath, emerge, morph, refusal } from "./glide-island-motion";

/** The black's size and corner radius, in px. */
export type GlideIslandShape = {
  height: number;
  radius: number;
  width: number;
};

type BodyProps = {
  children: ReactNode;
  /** Counts the rests completed; each one plays the ready breath. */
  pulse: number;
  /** The window will not move; the black shakes as it comes out. */
  refuses: boolean;
  shape: GlideIslandShape;
  /** Names what the island shows, for assistive technology. Leave it out
   * where something round several islands names them once. */
  label?: string;
};

// The island's black, in the recording island's colour and shadow, centred
// on where the gesture began. It changes shape on a spring and what it holds
// is laid over it, centred, so a look can swap for another while the black
// moves round both.

export function GlideIslandBody({
  isShown,
  ...props
}: BodyProps & { isShown: boolean }) {
  return (
    <AnimatePresence>
      {isShown ? (
        <motion.div
          key="island"
          {...emerge}
          className="dark pointer-events-none text-fg"
          // The shadow needs room inside the box that draws it: native WebKit
          // draws nothing of a filter outside the box. The room is taken back
          // out of the layout, so the black alone is centred.
          style={{ margin: -ISLAND.shadowRoom }}
        >
          <Black {...props} />
        </motion.div>
      ) : null}
    </AnimatePresence>
  );
}

function Black({ children, label, pulse, refuses, shape }: BodyProps) {
  const [blackRef, animate] = useAnimate<HTMLDivElement>();

  const firstPulseRef = useRef(pulse);
  useEffect(() => {
    if (pulse === firstPulseRef.current) return;
    void animate(blackRef.current, breath.keyframes, breath.transition);
  }, [animate, blackRef, pulse]);

  useEffect(() => {
    if (!refuses) return;
    void animate(blackRef.current, refusal.keyframes, refusal.transition);
  }, [animate, blackRef, refuses]);

  return (
    <div
      className="filter-(--island-shadow)"
      ref={blackRef}
      style={{ padding: ISLAND.shadowRoom }}
    >
      <motion.div
        animate={{
          borderRadius: shape.radius,
          height: shape.height,
          width: shape.width,
        }}
        aria-label={label}
        className="relative overflow-hidden bg-island"
        initial={false}
        role={label === undefined ? undefined : "img"}
        transition={morph}
      >
        {children}
      </motion.div>
    </div>
  );
}
