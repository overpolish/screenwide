// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";

import { motionDurationCss } from "../../lib/motion";
import { cn } from "../../lib/styling";

/** The fill behind one of the island's round controls: it grows in under the
 * pointer, deepens where the control is chosen or what it opens is open, and
 * shrinks away when it is left, so nothing on the island lights or goes out
 * at once. It sits behind the control's content, so the control is
 * `isolate`. Inside a button marked `group` the pointer is read from the
 * button's own hover, skipped while it is disabled; a control that is no
 * button says whether it is hovered. */
export function IslandFill({
  isChosen,
  isHovered,
}: {
  isChosen: boolean;
  isHovered?: boolean;
}) {
  const transitionDuration = useReducedMotion()
    ? "0s"
    : motionDurationCss("travel");
  return (
    <span
      aria-hidden
      className={cn(
        "pointer-events-none absolute inset-0 -z-10 rounded-full transition-[background-color,opacity,scale] ease-out",
        isChosen
          ? "bg-fill-strong"
          : isHovered === undefined
            ? "scale-50 bg-fill opacity-0 group-enabled:group-hover:scale-100 group-enabled:group-hover:opacity-100"
            : isHovered
              ? "bg-fill"
              : "scale-50 bg-fill opacity-0",
      )}
      style={{ transitionDuration }}
    />
  );
}
