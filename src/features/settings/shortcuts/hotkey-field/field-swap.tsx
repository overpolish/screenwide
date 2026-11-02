// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";
import { type ReactNode, useState } from "react";

import { glide } from "../../../../primitives/island/island-motion";
import { useSize } from "../../../../primitives/island/island-size";
import { IslandSwap } from "../../../../primitives/island/island-swap";

const still = { duration: 0 };

/** The field's place in its row, as it swaps between the shortcut's button
 * and the key in hand with its modifiers. The two swap as an island's parts
 * do (`IslandSwap`), and the place glides to the width of the one arriving,
 * so what sits beside it moves over rather than jumping, before the new one
 * has cleared in. */
export function FieldSwap({
  children,
  swapKey,
}: {
  children: ReactNode;
  swapKey: string;
}) {
  const reduce = useReducedMotion() ?? false;
  const [content, setContent] = useState<HTMLDivElement | null>(null);
  const size = useSize(content);
  return (
    // Positioned, so the one leaving is lifted out of the flow here.
    <motion.div
      animate={size ? { width: size.width } : undefined}
      className="relative flex justify-end"
      initial={false}
      transition={reduce ? still : glide}
    >
      <IslandSwap
        className="w-max shrink-0"
        initial={false}
        // The one leaving lets go of the ref as it goes, after the one
        // arriving has taken it.
        ref={(element) => {
          if (element) setContent(element);
        }}
        swapKey={swapKey}
      >
        {children}
      </IslandSwap>
    </motion.div>
  );
}
