// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RotateCcw } from "lucide-react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { type ReactNode } from "react";

import { IconButton } from "../../../primitives/button/icon-button";
import { focusIn } from "../../../primitives/island/island-motion";

/** A figure, after its reset while there is anything to put back. The
 * reset's place is kept while it is gone, so it comes into focus there, as
 * the island's parts do, without the row, the group or the island's outline
 * changing width round it. */
export function IslandFigure({
  children,
  controlProps,
  isDefault,
  onReset,
  resetLabel,
}: {
  children: ReactNode;
  controlProps: object;
  isDefault: boolean;
  onReset: () => void;
  resetLabel: string;
}) {
  const reduce = useReducedMotion();
  return (
    <div
      {...controlProps}
      className="flex items-center gap-inline"
      role="group"
    >
      <span className="flex size-island-control">
        <AnimatePresence initial={false}>
          {isDefault ? null : (
            <motion.span
              animate={focusIn.animate}
              className="flex"
              exit={focusIn.exit}
              initial={focusIn.exit}
              key="reset"
              transition={reduce ? { duration: 0 } : focusIn.transition}
            >
              <IconButton
                aria-label={resetLabel}
                className="text-fg-secondary enabled:hover:text-fg"
                onPress={onReset}
              >
                <RotateCcw aria-hidden className="size-icon" />
              </IconButton>
            </motion.span>
          )}
        </AnimatePresence>
      </span>
      {children}
    </div>
  );
}
