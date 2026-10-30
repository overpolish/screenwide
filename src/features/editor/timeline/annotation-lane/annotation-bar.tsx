// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";
import { useRef } from "react";

import { t } from "../../../../i18n/i18n";
import { motionDurations, motionEasings } from "../../../../lib/motion";
import { cn } from "../../../../lib/styling";
import { Badge } from "../../../../primitives/badge/badge";
import { annotationToolLabel } from "../../../annotations/annotation-text";
import { ANNOTATION_TOOLS } from "../../tool-panels/tool-registry";
import { reach, type Span, type TimelineScale } from "../timeline-scale";

import {
  type AnnotationBarPart,
  type BarHold,
  type Snap,
  useAnnotationBarDrag,
} from "./use-annotation-bar-drag";

import type { AnnotationKind } from "../../../../bindings/AnnotationKind";

// One annotation across the time it shows: a hairline with the badge of the
// tool that drew it in its middle, however short it is. Chosen, the line
// thickens and grows a handle at each end. The hit area is taller and wider
// than the line drawn, so a short annotation is still easy to take hold of.
// It is placed in a strip `width` px wide, which it measures the pointer
// against while held (`use-annotation-bar-drag.ts`), and glides to a new
// `top` unless it is being carried there.

/** A bar's height, in px: a badge's, with a little room. */
export const ANNOTATION_BAR_PX = 20;

/** The line stops short of the badge in its middle, by the badge's width
 * with a little air, so the badge's own fill never shows the line through. */
const GAP_UNDER_BADGE = `linear-gradient(to right, #000 calc(50% - 12px), transparent calc(50% - 12px), transparent calc(50% + 12px), #000 calc(50% + 12px))`;
/** How far, in px, a chosen bar's handles stand from its middle at least:
 * clear of the badge, so a bar shorter than that still shows its tool. */
const HANDLE_CLEAR_PX = 19;

export type AnnotationBarProps = {
  isSelected: boolean;
  kind: AnnotationKind;
  onRetime: (start: number, end: number) => void;
  onSelect: () => void;
  scale: TimelineScale;
  span: Span;
  /** The strip's width, in px. */
  width: number;
  /** Whether it is being dragged up or down, following the pointer rather
   * than gliding to its `top`. */
  isLifted?: boolean;
  /** Told each move of a drag, and null when it is let go; without it, the
   * bar only moves in time. */
  onHold?: (hold: BarHold | null) => void;
  /** Where a dragged end settles while ⌘ (Ctrl on Windows) is held;
   * without it, ends go where they are let go. */
  snap?: Snap;
  /** Where it sits, in px from the top of the strip. */
  top?: number;
};

export function AnnotationBar({
  isLifted = false,
  isSelected,
  kind,
  onHold,
  onRetime,
  onSelect,
  scale,
  snap,
  span,
  top = 0,
  width,
}: AnnotationBarProps) {
  const ref = useRef<HTMLDivElement>(null);
  const reducedMotion = useReducedMotion() ?? false;
  const { drag, grab, guide, release } = useAnnotationBarDrag({
    isSelected,
    onHold,
    onRetime,
    onSelect,
    scale,
    snap,
    span,
  });
  const { pad, px, style } = reach(scale, span, width);
  // Each handle at its end, or clear of the badge where the bar is shorter.
  const handleReach = Math.max(px / 2, HANDLE_CLEAR_PX);
  const take = (
    event: React.PointerEvent<HTMLElement>,
    part: AnnotationBarPart,
  ) => {
    // Held by the bar, the press never reaches the strip under it.
    event.stopPropagation();
    if (ref.current) grab(event, ref.current, part);
  };
  return (
    <motion.div
      animate={{ top }}
      aria-label={t("editor-timeline-annotation", {
        tool: annotationToolLabel(kind),
      })}
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
      style={{ ...style, height: ANNOTATION_BAR_PX }}
      tabIndex={-1}
      transition={
        isLifted || reducedMotion
          ? { duration: 0 }
          : { duration: motionDurations.travel, ease: motionEasings.out }
      }
    >
      <span
        className={cn(
          "absolute top-1/2 -translate-y-1/2 rounded-full transition-[height,background-color]",
          isSelected
            ? "h-1.5 bg-primary"
            : "h-0.5 bg-fg-tertiary group-hover:h-1 group-hover:bg-fg-secondary",
        )}
        style={{
          insetInlineEnd: pad,
          insetInlineStart: pad,
          maskImage: GAP_UNDER_BADGE,
        }}
      />
      <span
        aria-hidden
        className="absolute start-1/2 top-1/2 flex -translate-1/2"
      >
        <Badge
          className={cn(
            "px-0.5 text-surface transition-colors",
            isSelected
              ? "bg-primary text-on-primary"
              : "bg-fg-tertiary group-hover:bg-fg-secondary",
          )}
        >
          <ToolGlyph kind={kind} />
        </Badge>
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
      {guide === null ? null : (
        // Down through the beats under the lane, to the event an end holds to.
        <span
          className="pointer-events-none absolute top-0 h-24 w-px -translate-x-1/2 border-s border-dashed border-fg-secondary"
          style={{
            insetInlineStart:
              pad + (scale.toX(guide) - scale.toX(span.start)) * width,
          }}
        />
      )}
    </motion.div>
  );
}

/** The glyph of the tool that drew an annotation, as the tool column has it. */
function ToolGlyph({ kind }: { kind: AnnotationKind }) {
  const Icon = ANNOTATION_TOOLS.find(({ id }) => id === kind)?.icon;
  return Icon ? <Icon /> : null;
}
