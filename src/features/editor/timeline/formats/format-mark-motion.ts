// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";

import { motionDurations, motionEasings } from "../../../../lib/motion";

const away = { opacity: 0, scale: 0 };

/** How a mark saying something differs between formats comes and goes,
 * inside `AnimatePresence`: growing in and shrinking away rather than
 * cutting, at once where motion is reduced. Where it sits is the mark's
 * own to set, at once. `transition` is the same timing, for room that
 * opens and closes round a mark. */
export function useFormatMarkMotion() {
  const reducedMotion = useReducedMotion() ?? false;
  const transition = {
    duration: reducedMotion ? 0 : motionDurations.travel,
    ease: motionEasings.out,
  };
  return {
    mark: () => ({
      animate: { opacity: 1, scale: 1 },
      exit: away,
      initial: away,
      transition,
    }),
    transition,
  };
}
