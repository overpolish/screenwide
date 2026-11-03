// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { CSSProperties, ReactNode } from "react";

import { cn } from "../../../lib/styling";

// A line down the beats with a small glyph at one end, pressed to undo what
// it marks: a cut's seam, a split, a jump. Its glyph says what it undoes and
// its end says which kind it is, so marks that meet still read apart.

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
    <button
      aria-label={label}
      className={cn(
        "group absolute inset-y-0 z-10 flex w-4 -translate-x-1/2 cursor-default justify-center outline-none",
        end === "top" ? "items-start" : "items-end",
      )}
      dir="ltr"
      onClick={onPress}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      style={style}
      type="button"
    >
      <span
        className={cn(
          "absolute inset-y-0 w-px",
          isDashed
            ? "border-s border-dashed border-fg-secondary"
            : "bg-fg-secondary",
        )}
      />
      <span
        className={cn(
          "relative flex size-4 items-center justify-center rounded-full bg-fg text-surface group-focus-visible:outline-2 group-focus-visible:outline-offset-1 group-focus-visible:outline-focus [&_svg]:size-icon-xs",
          end === "top" ? "-mt-1" : "-mb-1",
        )}
      >
        {glyph}
      </span>
    </button>
  );
}
