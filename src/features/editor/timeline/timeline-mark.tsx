// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { CSSProperties, ReactNode } from "react";

import { cn } from "../../../lib/styling";

// A line down the beats with a small glyph at one end, which is pressed to
// undo what the line marks: a cut's seam, a split, a jump. Its glyph says
// what it undoes and its end says which kind it is, so marks that meet
// still read apart.

export function TimelineMark({
  end,
  glyph,
  isDashed = false,
  label,
  onPress,
  style,
}: {
  /** Which end of the beats the glyph sits at. */
  end: "bottom" | "top";
  glyph: ReactNode;
  label: string;
  onPress: () => void;
  /** Where along the timeline it stands (`at`). */
  style: CSSProperties;
  isDashed?: boolean;
}) {
  return (
    // Only the glyph is pressed: the line lets presses through to the beats
    // under it, so a narrow one beside a mark can still be chosen.
    <span
      className={cn(
        "pointer-events-none absolute inset-y-0 z-10 flex w-4 -translate-x-1/2 justify-center",
        end === "top" ? "items-start" : "items-end",
      )}
      dir="ltr"
      style={style}
    >
      <span
        aria-hidden
        className={cn(
          "absolute inset-y-0 w-px",
          isDashed
            ? "border-s border-dashed border-fg-secondary"
            : "bg-fg-secondary",
        )}
      />
      <button
        aria-label={label}
        className={cn(
          "pointer-events-auto relative flex size-4 cursor-default items-center justify-center rounded-full bg-fg text-surface outline-none focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-focus [&_svg]:size-icon-xs",
          end === "top" ? "-mt-1" : "-mb-1",
        )}
        onClick={onPress}
        onPointerDown={(event) => {
          event.stopPropagation();
        }}
        type="button"
      >
        {glyph}
      </button>
    </span>
  );
}
