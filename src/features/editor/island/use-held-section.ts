// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";
import { useEffect, useState } from "react";

import { glide, leave } from "../../../primitives/island/island-motion";

/**
 * Whether the island should keep a section that has just gone while the
 * same thing stays in hand, such as one an option turned off took away: it
 * fades and folds away as the island's parts do (`IslandParts`), with any
 * part of the row going with it, before the island lets its room go. With
 * something else in hand the row and section change over together instead.
 */
export function useHeldSection(key: string, hasSection: boolean) {
  const reduce = useReducedMotion() ?? false;
  const [last, setLast] = useState({ hasSection, key });
  const [held, setHeld] = useState<string | null>(null);
  if (last.key !== key || last.hasSection !== hasSection) {
    setLast({ hasSection, key });
    setHeld(
      !reduce && last.key === key && last.hasSection && !hasSection
        ? key
        : null,
    );
  }
  // Let go once the fold is done and the black glides again, so the box's
  // own inset closes on the black's curve rather than at once.
  useEffect(() => {
    if (held === null) return;
    const done = window.setTimeout(
      () => {
        setHeld(null);
      },
      (leave.duration + glide.duration) * 1000 + 60,
    );
    return () => {
      window.clearTimeout(done);
    };
  }, [held]);
  return held === key;
}
