// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../../lib/styling";

import { type BarWave, wavyLinePath } from "./annotation-bar-waves";

// Where an annotation stands somewhere of its own in a scene, its line makes
// one wave over that scene's stretch: it moves there. The bar's whole line is
// then this one stroke, so the wave runs on from the straight line without a
// seam, in the line's colour and weight. It reads apart from a pin's hatch
// and ticks, which mark the line rather than change it. Only decorates; the
// annotation's island says it has a place in the scene at the playhead.

export function AnnotationBarWavyLine({
  height,
  isSelected,
  length,
  mask,
  pad,
  waves,
}: {
  /** The bar's height, in px, whose middle the line runs along. */
  height: number;
  isSelected: boolean;
  /** The line's length, in px. */
  length: number;
  /** The line's gap under the badge. */
  mask: string;
  /** How far, in px, the line starts into the bar's box. */
  pad: number;
  waves: BarWave[];
}) {
  return (
    <svg
      aria-hidden
      className={cn(
        "pointer-events-none absolute inset-y-0 overflow-visible transition-[stroke-width,color]",
        isSelected
          ? "stroke-[6] text-primary"
          : "stroke-2 text-fg-tertiary group-hover:stroke-[4] group-hover:text-fg-secondary",
      )}
      height={height}
      style={{ insetInlineStart: pad, maskImage: mask }}
      width={length}
    >
      <path
        d={wavyLinePath(length, waves, height / 2)}
        fill="none"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
