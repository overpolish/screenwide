// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Check } from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import { type ReactNode } from "react";
import { Button } from "react-aria-components";

import { useConfirmAction } from "../../../../composites/confirm-action-button/use-confirm-action";
import { cn } from "../../../../lib/styling";
import { IslandFill } from "../../../../primitives/island/island-fill";

// One of the island's round controls that asks twice, on the window's own
// confirm logic (`use-confirm-action.ts`): the first press arms it, filled
// in the primary colour with a check, and a second within a moment does it.
// Leaving it, Escape or the moment passing disarms it.

export function IslandConfirmButton({
  children,
  confirmLabel,
  label,
  onConfirm,
}: {
  /** The glyph at rest. */
  children: ReactNode;
  /** What the second press does, said while armed. */
  confirmLabel: string;
  label: string;
  onConfirm: () => void;
}) {
  const { buttonProps, isArmed } = useConfirmAction({ onConfirm });
  const { className, ...pressProps } = buttonProps;
  return (
    <Button
      {...pressProps}
      aria-label={isArmed ? confirmLabel : label}
      className={cn(
        "relative isolate flex size-island-control shrink-0 items-center justify-center rounded-full transition-colors",
        isArmed && "bg-primary text-on-primary",
        className,
      )}
    >
      {({ isHovered }) => (
        <>
          {isArmed ? null : (
            <IslandFill isChosen={false} isHovered={isHovered} />
          )}
          <span className="grid">
            <AnimatePresence initial={false}>
              <motion.span
                animate={{ opacity: 1 }}
                aria-hidden
                className="col-start-1 row-start-1 flex items-center justify-center"
                exit={{ opacity: 0 }}
                initial={{ opacity: 0 }}
                key={isArmed ? "armed" : "idle"}
              >
                {isArmed ? <Check className="size-icon" /> : children}
              </motion.span>
            </AnimatePresence>
          </span>
        </>
      )}
    </Button>
  );
}
