// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";
import { Fragment, type ReactNode, useEffect, useRef, useState } from "react";

import { motionDurations, motionEasings } from "../../../../lib/motion";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { SnapGuide } from "../strip/snap-guide";
import { baseOf } from "../strip/strip-part";
import { type SnapFor } from "../strip/strip-snap";
import { type Span, type TimelineScale } from "../timeline-scale";

import { type StackBarPlacement } from "./stack-bar";
import { carryScrollSpeed, type Held, heldLanes, holdStep } from "./stack-hold";
import { BAR_PX, packLanes, type StackedBar } from "./stack-lanes";
import { type IncomingBar, useIncomingBar } from "./use-incoming-bar";
import { type BarPart } from "./use-stack-bar-drag";

// Bars across the time each spans, stacked in lanes; the lanes are the
// stacking order, the top one drawn in front, and magnetic: each bar rises
// as high as that order lets it (`stack-lanes.ts`). Dragged, a bar follows
// the pointer in time and up and down, between just above the top lane and
// just below the bottom one, and the others glide to where they will sit
// when it is let go: into the lane under it where it fits, or a new lane
// opened there; the slots hold steady near their edges (`stack-hold.ts`).
// Kept to its lane with Shift, or held by an end, a bar moved onto another
// in its lane takes a new lane of its own, above or below as the pointer
// is, and goes back once it is clear; the others rise into any room it
// leaves. With ⌘ held, ends snap to what `snap` offers, the strip's one
// list of edges (`strip-snap.ts`), which leaves out the bar's own. The
// lanes grow to `MAX_SHOWN_LANES`; past that they scroll inside that
// height, and the rest of the strip stays put. A bar carried near their top
// or bottom scrolls them on, staying under the pointer. What each bar shows
// is the stack's owner's (`renderBar`).

/** The most lanes shown at once; past them the lanes scroll. */
const MAX_SHOWN_LANES = 4;

/** How far past the lanes a snapped end's guide reaches, through the beats
 * to the event it holds to, in px. */
const GUIDE_PAST_PX = 72;

export type BarStackProps<T extends StackedBar> = {
  bars: T[];
  /** Which way the beats are, which a snapped end's guide reaches to. */
  guideTo: "above" | "below";
  /** Told every bar's lane, by id, after a drag moved any. */
  onRestack: (lanes: Record<string, number>) => void;
  /** Told a bar's new span, and whether the whole bar or an end moved. */
  onRetime: (id: string, span: Span, part: BarPart) => void;
  onSelect: (id: string) => void;
  renderBar: (bar: T, placement: StackBarPlacement) => ReactNode;
  scale: TimelineScale;
  /** The strip's width, in px. */
  width: number;
  /** A bar brought in from outside, such as a picture dragged in, carried
   * with the pointer as a bar held is (`use-incoming-bar.ts`). */
  incoming?: IncomingBar<T> | null;
  /** Told every bar's lane if the incoming one is let go now, or null. */
  onIncomingLanes?: (lanes: Record<string, number> | null) => void;
  selected?: string;
  /** Where a dragged end settles while ⌘ is held, given its bar, whose own
   * edges are left out. */
  snap?: SnapFor;
};

export function BarStack<T extends StackedBar>({
  bars,
  guideTo,
  incoming = null,
  onIncomingLanes,
  onRestack,
  onRetime,
  onSelect,
  renderBar,
  scale,
  selected,
  snap,
  width,
}: BarStackProps<T>) {
  const reducedMotion = useReducedMotion() ?? false;
  const [held, setHeld] = useState<Held | null>(null);
  const [viewport, setViewport] = useState<HTMLElement | null>(null);
  const [scrollTop, setScrollTop] = useState(0);
  // Lanes are about what shows at once as it plays, so they are packed on
  // where the bars are drawn, which a reordered recording no longer keeps in
  // recorded order.
  const drawnSpan = (span: Span) => ({
    end: scale.toX(span.end, true),
    start: scale.toX(span.start),
  });
  const drawn = bars.map((each) => ({ ...each, ...drawnSpan(each) }));
  // The lanes risen, as the strip shows them with nothing held; held, where
  // every bar goes if it is let go now.
  const settled = packLanes(drawn);
  const coming = useIncomingBar({
    drawn,
    drawnSpan,
    incoming,
    onLanes: onIncomingLanes,
    scrollTop,
    settled,
  });
  const active = held ?? coming.held;
  const lanes = held ? heldLanes(drawn, held) : (coming.lanes ?? settled);
  const laneOf = (bar: T) => lanes[bar.id];
  const shownCount = Math.max(
    1,
    ...bars.map((bar) => laneOf(bar) + 1),
    ...(active?.liftedTop == null
      ? []
      : [Math.ceil(active.liftedTop / BAR_PX + 0.5)]),
  );
  // Only the rows the bars use: a chosen bar's handles, a little taller
  // than its row, show in the scrolling box's margin.
  const contentHeight = shownCount * BAR_PX;
  const height = Math.min(contentHeight, MAX_SHOWN_LANES * BAR_PX);
  useEffect(() => {
    if (!viewport) return;
    const onScroll = () => {
      setScrollTop(viewport.scrollTop);
    };
    // While the lanes have more to show, an upright wheel scrolls them
    // rather than moving along the strip (`use-strip-view.ts`).
    const onWheel = (event: WheelEvent) => {
      if (event.ctrlKey || event.metaKey) return;
      if (Math.abs(event.deltaY) <= Math.abs(event.deltaX)) return;
      if (viewport.scrollHeight > viewport.clientHeight)
        event.stopPropagation();
    };
    viewport.addEventListener("scroll", onScroll, { passive: true });
    viewport.addEventListener("wheel", onWheel, { passive: true });
    return () => {
      viewport.removeEventListener("scroll", onScroll);
      viewport.removeEventListener("wheel", onWheel);
    };
  }, [viewport]);
  // Where the pointer is down the lanes shown, while a bar is held.
  const pointerRef = useRef(0);
  const isCarried = held?.liftedTop != null;
  // Only lanes past the most shown scroll. Fewer, they grow to fit, and the
  // moment they grow under a carried bar never reads as more to scroll:
  // no scrollbar, no fade, and no carrying the lanes along.
  const isScrolling = contentHeight > MAX_SHOWN_LANES * BAR_PX;
  useEffect(() => {
    if (!viewport || !isCarried || !isScrolling) return;
    let frame = 0;
    let last: number | null = null;
    // Kept unrounded, so a slow scroll still moves.
    let position = viewport.scrollTop;
    const step = (now: number) => {
      const speed = carryScrollSpeed(pointerRef.current, height);
      if (Math.abs(viewport.scrollTop - position) >= 1)
        position = viewport.scrollTop;
      if (last !== null && speed !== 0) {
        const end = viewport.scrollHeight - viewport.clientHeight;
        position = Math.min(
          Math.max(position + speed * Math.min(now - last, 50), 0),
          end,
        );
        viewport.scrollTop = position;
        // Told at once, the held bar moves in this frame along with the
        // lanes, rather than a frame behind them when the scroll is told.
        viewport.dispatchEvent(new Event("scroll"));
      }
      last = now;
      frame = requestAnimationFrame(step);
    };
    frame = requestAnimationFrame(step);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [height, isCarried, isScrolling, viewport]);
  // Lanes that all fit are never left scrolled by a moment they did not.
  useEffect(() => {
    if (viewport && !isScrolling && viewport.scrollTop !== 0) {
      viewport.scrollTop = 0;
      viewport.dispatchEvent(new Event("scroll"));
    }
  });
  // Drawn over the lanes rather than in them, so it is not cut off where
  // they scroll: from the held bar, as far as it shows, past them to the
  // beats.
  const guideTop =
    held?.guide == null
      ? null
      : Math.min(
          Math.max((held.liftedTop ?? lanes[held.id] * BAR_PX) - scrollTop, 0),
          height,
        );

  return (
    <motion.div
      animate={{ height }}
      className="relative"
      dir="ltr"
      initial={false}
      transition={{
        duration: reducedMotion ? 0 : motionDurations.travel,
        ease: motionEasings.out,
      }}
    >
      {/* The scrolling box reaches past the lanes' own, so what is drawn a
        little outside them still shows: a chosen bar's handles at the
        strip's ends, a bar carried half a lane above or below them. That
        margin lets presses through to the strip under it. */}
      <ScrollArea
        className="px-2 py-2.5"
        edgeEffect={isScrolling ? "fade" : "none"}
        onInitialized={setViewport}
        rootClassName="pointer-events-none absolute -inset-x-2 -inset-y-2.5 h-auto w-auto"
        scrollbarHidden={!isScrolling}
      >
        <div
          className="pointer-events-auto relative"
          style={{ height: contentHeight }}
        >
          {bars.map((bar) => {
            const liftedTop = bar.id === held?.id ? held.liftedTop : null;
            return (
              <Fragment key={bar.id}>
                {renderBar(bar, {
                  isLifted: liftedTop !== null,
                  isSelected: baseOf(bar.id) === selected,
                  onHold: (hold) => {
                    if (hold) {
                      pointerRef.current = hold.y - (viewport?.scrollTop ?? 0);
                      setHeld((current) =>
                        holdStep({
                          bars: drawn,
                          current,
                          // Where the bar is placed, not where the pointer
                          // has it: its owner keeps it to its limits, such
                          // as a sound's end, and lanes made for a span it
                          // never takes would jump with every move.
                          hold: { ...hold, span: drawnSpan(bar) },
                          id: bar.id,
                          shown: settled,
                        }),
                      );
                      return;
                    }
                    // Every move renders the stack again, so the bar is let
                    // go with this render's handler and its lanes.
                    setHeld(null);
                    if (bars.some(({ id, lane }) => lanes[id] !== lane))
                      onRestack(lanes);
                  },
                  onRetime: (start, end, part) => {
                    onRetime(bar.id, { end, start }, part);
                  },
                  onSelect: () => {
                    onSelect(bar.id);
                  },
                  scale,
                  snap: snap?.(baseOf(bar.id)),
                  span: bar,
                  top: liftedTop ?? laneOf(bar) * BAR_PX,
                  width,
                })}
              </Fragment>
            );
          })}
          {incoming && coming.held
            ? renderBar(incoming.bar, {
                isLifted: true,
                isSelected: false,
                onHold: () => {},
                onRetime: () => {},
                onSelect: () => {},
                scale,
                span: incoming.bar,
                top: coming.held.liftedTop ?? 0,
                width,
              })
            : null}
        </div>
      </ScrollArea>
      {guideTop === null || held?.guide == null ? null : (
        <SnapGuide
          style={
            guideTo === "below"
              ? {
                  height: height - guideTop + GUIDE_PAST_PX,
                  insetInlineStart: scale.toX(held.guide) * width,
                  top: guideTop,
                }
              : {
                  height: guideTop + BAR_PX + GUIDE_PAST_PX,
                  insetInlineStart: scale.toX(held.guide) * width,
                  top: -GUIDE_PAST_PX,
                }
          }
        />
      )}
    </motion.div>
  );
}
