// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion } from "motion/react";

import { motionDurations } from "../../lib/motion";
import { cn } from "../../lib/styling";

import { px } from "./island-geometry";

/** The inner corner on one side of the section: the island's black where the
 * row and the section meet, cut by a quarter circle on the open side. It
 * sits in the section's box at the row's bottom, and its curve shrinks with
 * the black in step with the black's own corners (`islandCurves`), so it
 * grows out and drains away with the section. It fades of its own only
 * where the section meets the row's end, where the row's corner is square. */
export function Fillet({
  isShown,
  radius,
  side,
  top,
}: {
  isShown: boolean;
  /** The curve's radius, a CSS length. */
  radius: string;
  side: "start" | "end";
  /** The row's bottom, in px down the section's box. */
  top: number;
}) {
  return (
    <motion.div
      animate={{ opacity: isShown ? 1 : 0 }}
      aria-hidden
      className={cn(
        // A pixel taller and wider than the curve: the top one is tucked
        // under the row, so the curve starts on the row's edge with no step
        // or seam, and the side one overlaps the section's edge. The
        // curve's edge is softened by half a pixel.
        "pointer-events-none absolute size-[calc(var(--fillet)+1px)]",
        side === "start"
          ? "-start-(--fillet) bg-[radial-gradient(circle_at_0_100%,transparent_calc(var(--fillet)-0.5px),var(--color-island)_var(--fillet))] rtl:bg-[radial-gradient(circle_at_100%_100%,transparent_calc(var(--fillet)-0.5px),var(--color-island)_var(--fillet))]"
          : "-end-(--fillet) bg-[radial-gradient(circle_at_100%_100%,transparent_calc(var(--fillet)-0.5px),var(--color-island)_var(--fillet))] rtl:bg-[radial-gradient(circle_at_0_100%,transparent_calc(var(--fillet)-0.5px),var(--color-island)_var(--fillet))]",
      )}
      initial={false}
      style={{ "--fillet": radius, top: px(top - 1) } as React.CSSProperties}
      transition={
        isShown
          ? { delay: motionDurations.feedback, duration: motionDurations.state }
          : { duration: motionDurations.feedback }
      }
    />
  );
}
