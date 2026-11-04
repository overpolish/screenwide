// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";
import { useEffect, useState } from "react";

import { leave } from "./island-motion";

/**
 * `value`, taken up only once what the island showed for the last one has
 * faded where it stood (`leave`, as `IslandParts` fades a part), so the
 * black reshapes round what comes next in one move, rather than carrying
 * what is going away with it. At once while the island is away, and with
 * reduced motion.
 */
export function useAfterLeave<T>(value: T, { isShown }: { isShown: boolean }) {
  const reduce = useReducedMotion() ?? false;
  const [settled, setSettled] = useState(value);
  if ((!isShown || reduce) && settled !== value) setSettled(value);
  useEffect(() => {
    if (settled === value) return;
    const taken = window.setTimeout(() => {
      setSettled(value);
    }, leave.duration * 1000);
    return () => {
      window.clearTimeout(taken);
    };
  }, [value, settled]);
  return settled;
}
