// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  animate,
  motion,
  useMotionValue,
  useReducedMotion,
} from "motion/react";
import { type ReactNode, useLayoutEffect, useRef, useState } from "react";

import { motionDurations, motionEasings } from "../../../../lib/motion";
import { cn } from "../../../../lib/styling";
import { useSize } from "../../../../primitives/island/island-size";
import { reach, type Span, type TimelineScale } from "../timeline-scale";

import { BAR_PX } from "./stack-lanes";
import {
  type BarHold,
  type BarPart,
  type Snap,
  useStackBarDrag,
} from "./use-stack-bar-drag";

// One bar in a stack across the time it spans: a hairline with a badge in
// its middle, however short it is, the line stopping short of the badge.
// Chosen, the line thickens and grows a handle at each end. The hit area is
// taller and wider than the line drawn, so a short bar is still easy to
// take hold of. It is placed in a strip `width` px wide, which it measures
// the pointer against while held (`use-stack-bar-drag.ts`), and glides to a
// new `top` unless it is being carried there. What the badge holds, and
// anything drawn along the line, is the stack's own.

/** Room, in px, the line leaves either side of the badge. */
const BADGE_AIR_PX = 3;
/** How far past the badge's half, in px, a chosen bar's handles stand at
 * least, so a bar shorter than that still shows its badge. */
const HANDLE_CLEAR_PX = 7;
/** How far, in px, the stack's owner has to move a carried bar away from
 * where the pointer has it for the bar to glide there. */
const JUMP_PX = 8;

/** Where the line is drawn: `mask` leaves the badge clear, `pad` is how far
 * the bar's box reaches past its span at each end, `px` its drawn width. */
export type StackBarLook = { mask: string; pad: number; px: number };

/** A mask that hides the line `half` px either side of its middle, fading
 * it out over `fade` px before that, as a lull's wire fades round its
 * glyphs, or cutting it where `fade` is 0. */
const gapUnderBadge = (half: number, fade: number) =>
  `linear-gradient(to right, #000 calc(50% - ${String(half + fade)}px), transparent calc(50% - ${String(half)}px), transparent calc(50% + ${String(half)}px), #000 calc(50% + ${String(half + fade)}px))`;

/** What the stack hands each bar: where it is, and what holding it does. */
export type StackBarPlacement = {
  isLifted: boolean;
  isSelected: boolean;
  /** Each move of a drag, then null. */
  onHold: (hold: BarHold | null) => void;
  /** Told the bar's new span, and whether the whole bar or an end moved. */
  onRetime: (start: number, end: number, part: BarPart) => void;
  onSelect: () => void;
  scale: TimelineScale;
  span: Span;
  /** Where it sits, in px from the top of the stack. */
  top: number;
  /** The strip's width, in px. */
  width: number;
  /** Never carried up and down, only along in time. */
  movesInTimeOnly?: boolean;
  /** Where a dragged end settles while ⌘ (Ctrl on Windows) is held. */
  snap?: Snap;
};

export type StackBarProps = StackBarPlacement & {
  /** What sits in the middle of the bar, given its drawn width in px. */
  badge: (px: number) => ReactNode;
  /** The bar's accessible name. */
  label: string;
  /** How far, in px, the line fades out before the badge; 0 cuts it. */
  badgeFade?: number;
  /** Draws the line, in place of the plain one (`StackBarLine`). */
  line?: (look: StackBarLook) => ReactNode;
};

/** The plain line: a hairline at rest, thicker under the pointer, and
 * thicker still and in the primary ink when chosen. */
export function StackBarLine({
  isSelected,
  look,
}: {
  isSelected: boolean;
  look: StackBarLook;
}) {
  return (
    <span
      className={cn(
        "absolute top-1/2 -translate-y-1/2 rounded-full transition-[height,background-color]",
        isSelected
          ? "h-1.5 bg-primary"
          : "h-0.5 bg-fg-tertiary group-hover:h-1 group-hover:bg-fg-secondary",
      )}
      style={{
        insetInlineEnd: look.pad,
        insetInlineStart: look.pad,
        maskImage: look.mask,
      }}
    />
  );
}

export function StackBar({
  badge,
  badgeFade = 0,
  isLifted,
  isSelected,
  label,
  line,
  movesInTimeOnly,
  onHold,
  onRetime,
  onSelect,
  scale,
  snap,
  span,
  top,
  width,
}: StackBarProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [badgeElement, setBadgeElement] = useState<HTMLSpanElement | null>(
    null,
  );
  const badgeWidth = useSize(badgeElement)?.width ?? 0;
  const reducedMotion = useReducedMotion() ?? false;
  // Where the pointer has the bar while it is held, against which a move
  // its owner made on its own shows.
  const pointerRef = useRef<Span | null>(null);
  const { drag, grab, release } = useStackBarDrag({
    isSelected,
    movesInTimeOnly,
    onHold: (hold) => {
      pointerRef.current = hold?.span ?? null;
      onHold(hold);
    },
    onRetime,
    onSelect,
    scale,
    snap,
    span,
  });
  const { pad, px, style } = reach(scale, span, width);
  // Held, a bar its owner moves on its own, such as a song jumping past
  // another, glides to where it lands rather than appearing there. Moves
  // that follow the pointer keep the bar where the pointer has it, and
  // never glide, so the bar never trails the pointer.
  const shift = useMotionValue(0);
  const pointer = pointerRef.current;
  const offset =
    pointer === null
      ? null
      : (scale.toX(span.start) - scale.toX(pointer.start)) * width;
  const offsetRef = useRef(offset);
  useLayoutEffect(() => {
    const was = offsetRef.current;
    offsetRef.current = offset;
    if (was === null || offset === null) return;
    const jump = was - offset;
    if (reducedMotion || Math.abs(jump) < JUMP_PX) return;
    shift.jump(shift.get() + jump);
    void animate(shift, 0, {
      duration: motionDurations.travel,
      ease: motionEasings.out,
    });
  }, [offset, reducedMotion, shift]);
  const badgeHalf = badgeWidth / 2 + BADGE_AIR_PX;
  const look: StackBarLook = {
    mask: gapUnderBadge(badgeHalf, badgeFade),
    pad,
    px,
  };
  // Each handle at its end, or clear of the badge where the bar is shorter.
  const handleReach = Math.max(px / 2, badgeHalf + HANDLE_CLEAR_PX);
  const take = (event: React.PointerEvent<HTMLElement>, part: BarPart) => {
    // Held by the bar, the press never reaches the strip under it.
    event.stopPropagation();
    if (ref.current) grab(event, ref.current, part);
  };
  return (
    <motion.div
      animate={{ top }}
      aria-label={label}
      aria-pressed={isSelected}
      className={cn(
        "group absolute cursor-grab touch-none active:cursor-grabbing",
        isLifted && "z-10 cursor-grabbing",
      )}
      dir="ltr"
      initial={false}
      onPointerDown={(event) => {
        take(event, "body");
      }}
      // A held end's moves and its release bubble up to here.
      onPointerMove={(event) => {
        drag(event);
      }}
      onPointerUp={release}
      ref={ref}
      role="button"
      style={{ ...style, height: BAR_PX, x: shift }}
      transition={
        isLifted || reducedMotion
          ? { duration: 0 }
          : { duration: motionDurations.travel, ease: motionEasings.out }
      }
    >
      {line ? line(look) : <StackBarLine isSelected={isSelected} look={look} />}
      <span
        aria-hidden
        className="absolute start-1/2 top-1/2 flex -translate-1/2"
        ref={setBadgeElement}
      >
        {badge(px)}
      </span>
      {isSelected
        ? (["start", "end"] as const).map((part) => (
            <span
              aria-hidden
              className="absolute top-1/2 flex size-6 -translate-1/2 cursor-ew-resize items-center justify-center"
              key={part}
              onPointerDown={(event) => {
                take(event, part);
              }}
              style={{
                insetInlineStart: `calc(50% ${part === "start" ? "-" : "+"} ${String(handleReach)}px)`,
              }}
            >
              <span className="size-3 rounded-full bg-primary ring-2 ring-recess" />
            </span>
          ))
        : null}
    </motion.div>
  );
}
