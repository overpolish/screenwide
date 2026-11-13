// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import { cn } from "../../../../lib/styling";
import { reach, type Span, type TimelineScale } from "../timeline-scale";

// One block on a rail of the strip over the stretch it holds, as a scene or
// a song is. What it shows grows with its width, as a beat's does: bare,
// then its glyph, then its glyph and name, cut short where it runs out of
// room. Pressed, it is chosen and its body can be dragged at once; chosen,
// it fills and grows a handle at each end. One a few pixels wide is
// pressed across more than it is drawn.

/** How far, in px, a press on a block moves before it is a drag, so a click
 * that only chooses it never nudges it. */
export const RAIL_DRAG_PX = 3;
/** The narrowest a block shows its glyph at, in px: the widest glyph and
 * the padding either side. */
const GLYPH_PX = 30;
/** The narrowest a block shows its name beside its glyph at, in px. */
const NAME_PX = 72;

export type RailClipProps = {
  isSelected: boolean;
  // The moves and release that follow bubble up from what was grabbed.
  /** A press on the block's body, which chooses it and can move it. */
  onGrabBody: (event: React.PointerEvent) => void;
  /** A press on one of a chosen block's ends. */
  onGrabEdge: (event: React.PointerEvent, edge: "end" | "start") => void;
  onPress: () => void;
  scale: TimelineScale;
  /** The strip's width, in px. */
  width: number;
};

export function RailClip({
  glyph,
  isSelected,
  label,
  name,
  onGrabBody,
  onGrabEdge,
  onPress,
  scale,
  span,
  width,
}: RailClipProps & {
  glyph: ReactNode;
  /** Its accessible name. */
  label: string;
  /** What it is called, shown where there is room. */
  name: string;
  span: Span;
}) {
  const { pad, px, style } = reach(scale, span, width);
  return (
    <button
      aria-label={label}
      aria-pressed={isSelected}
      className="group absolute inset-y-0 cursor-grab touch-none outline-none active:cursor-grabbing"
      dir="ltr"
      // A keyboard press; a pointer's is taken at the press, below.
      onClick={onPress}
      onPointerDown={(event) => {
        event.stopPropagation();
        event.currentTarget.setPointerCapture(event.pointerId);
        onGrabBody(event);
      }}
      style={style}
      type="button"
    >
      <span
        className={cn(
          "absolute inset-y-0.5 flex items-center gap-1 overflow-hidden rounded-part text-caption whitespace-nowrap transition-colors group-focus-visible:outline-2 group-focus-visible:outline-focus",
          // Padding would hold a narrow block wider than its time.
          px >= GLYPH_PX && "px-1",
          isSelected
            ? "bg-primary text-on-primary"
            : "bg-fill-strong text-fg-secondary group-hover:bg-fg-tertiary group-hover:text-fg",
        )}
        style={{ insetInlineEnd: pad, insetInlineStart: pad }}
      >
        {px >= GLYPH_PX ? glyph : null}
        {px >= NAME_PX ? (
          <span className="truncate" dir="auto">
            {name}
          </span>
        ) : null}
      </span>
      {isSelected
        ? (["start", "end"] as const).map((edge) => (
            <span
              aria-hidden
              className="absolute inset-y-0 flex w-3 -translate-x-1/2 cursor-ew-resize items-center justify-center"
              key={edge}
              onPointerDown={(event) => {
                event.stopPropagation();
                event.currentTarget.setPointerCapture(event.pointerId);
                onGrabEdge(event, edge);
              }}
              style={{
                insetInlineStart:
                  edge === "start" ? pad : `calc(100% - ${String(pad)}px)`,
              }}
            >
              <span className="h-3.5 w-1 rounded-full bg-on-primary ring-2 ring-primary" />
            </span>
          ))
        : null}
    </button>
  );
}
