// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../lib/styling";

import type { ScrubStrip } from "../types";

/** One frame of a scrub strip, covering its box as the still does. One
 * picture holds every frame, so moving between them shows one already
 * loaded: the strip slides along behind the frame's window. */
export function StripFrame({
  className,
  frame,
  strip,
}: {
  frame: number;
  strip: ScrubStrip;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "absolute inset-0 flex items-center justify-center",
        className,
      )}
    >
      <span
        className={cn(
          "relative shrink-0 overflow-hidden",
          strip.frameWidth / strip.frameHeight >= 16 / 9 ? "h-full" : "w-full",
        )}
        // The strip is one picture with its frames left to right.
        dir="ltr"
        style={{
          aspectRatio: `${String(strip.frameWidth)} / ${String(strip.frameHeight)}`,
        }}
      >
        <img
          alt=""
          className="absolute start-0 top-0 h-full max-w-none"
          draggable={false}
          src={strip.src}
          style={{
            transform: `translateX(-${String((frame * 100) / strip.frames)}%)`,
            width: `${String(strip.frames * 100)}%`,
          }}
        />
      </span>
    </span>
  );
}
