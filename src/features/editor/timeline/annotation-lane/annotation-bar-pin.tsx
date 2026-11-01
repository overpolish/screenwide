// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../../lib/styling";
import { SQUEEZED_HATCH } from "../strip/taken-out-band";
import { type Span, type TimelineScale } from "../timeline-scale";

// What a pinned annotation's bar shows of its pin, under the badge: where
// its content is out of view, so it is hidden, hatched as what the squeeze
// took is, and each place it was put by hand as a tick across the line.
// Over the line, where the tracker is unsure of the content, the line
// itself is drawn in the warning colour, as a mark rather than a tinted
// surface, so it reads apart from the hatch and through a chosen bar's
// thicker line. All only decorate; the bar's own name says it is pinned,
// and the island says what is unsure at the playhead.

/** A pinned annotation's pin, as its bar draws it, in the strip's seconds
 * and inside the bar's span. */
export type TimelineAnnotationPin = {
  /** Each moment it was put somewhere by hand. */
  corrections: number[];
  /** Where its content is out of view, and it is hidden. */
  outOfView: Span[];
  /** Where the tracker is unsure of its content: lost, off the frame or
   * under a cover. */
  unsure: Span[];
};

/** Where the tracker is unsure, over the bar's line: the same reach and the
 * same gap under the badge (`mask`), a touch thicker than the line at rest
 * so the colour reads. */
export function AnnotationBarUnsure({
  isSelected,
  mask,
  pad,
  pin,
  scale,
  span,
  width,
}: {
  isSelected: boolean;
  /** The line's mask, which leaves the badge clear. */
  mask: string;
  /** How far, in px, the bar's box reaches past its span at each end. */
  pad: number;
  pin: TimelineAnnotationPin;
  scale: TimelineScale;
  span: Span;
  /** The strip's width, in px. */
  width: number;
}) {
  if (pin.unsure.length === 0) return null;
  // In px from the line's start.
  const along = (seconds: number, isEnd?: boolean) =>
    (scale.toX(seconds, isEnd) - scale.toX(span.start)) * width;
  return (
    <span
      aria-hidden
      className="pointer-events-none absolute inset-y-0"
      style={{ insetInlineEnd: pad, insetInlineStart: pad, maskImage: mask }}
    >
      {pin.unsure.map((stretch) => (
        <span
          className={cn(
            "absolute top-1/2 -translate-y-1/2 rounded-full bg-warning transition-[height]",
            isSelected ? "h-1.5" : "h-1",
          )}
          key={stretch.start}
          style={{
            insetInlineStart: along(stretch.start),
            width: Math.max(0, along(stretch.end, true) - along(stretch.start)),
          }}
        />
      ))}
    </span>
  );
}

export function AnnotationBarPin({
  isSelected,
  mask,
  pad,
  pin,
  scale,
  span,
  width,
}: {
  isSelected: boolean;
  /** The line's gap under the badge, which the badge's translucent fill would
   * otherwise show the hatch and the ticks through. */
  mask: string;
  /** How far, in px, the bar's box reaches past its span at each end. */
  pad: number;
  pin: TimelineAnnotationPin;
  scale: TimelineScale;
  span: Span;
  /** The strip's width, in px. */
  width: number;
}) {
  // In px from the bar's box's start.
  const along = (seconds: number, isEnd?: boolean) =>
    (scale.toX(seconds, isEnd) - scale.toX(span.start)) * width + pad;
  return (
    <span
      aria-hidden
      className="pointer-events-none absolute inset-0"
      style={{ maskImage: mask }}
    >
      {pin.outOfView.map((stretch) => (
        <span
          aria-hidden
          className={cn(
            "absolute top-1/2 h-2.5 -translate-y-1/2 rounded-sm",
            SQUEEZED_HATCH,
          )}
          key={stretch.start}
          style={{
            insetInlineStart: along(stretch.start),
            width: Math.max(0, along(stretch.end, true) - along(stretch.start)),
          }}
        />
      ))}
      {pin.corrections.map((moment) => (
        <span
          aria-hidden
          className={cn(
            "absolute top-1/2 h-2.5 w-px -translate-1/2 rounded-full transition-colors",
            isSelected
              ? "bg-primary"
              : "bg-fg-tertiary group-hover:bg-fg-secondary",
          )}
          key={moment}
          style={{ insetInlineStart: along(moment) }}
        />
      ))}
    </span>
  );
}
