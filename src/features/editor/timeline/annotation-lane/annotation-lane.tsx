// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";
import { useEffect, useRef, useState } from "react";

import { motionDurations, motionEasings } from "../../../../lib/motion";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { SnapGuide } from "../strip/snap-guide";
import { baseOf } from "../strip/strip-part";
import { type SnapFor } from "../strip/strip-snap";
import { type Span, type TimelineScale } from "../timeline-scale";

import { ANNOTATION_BAR_PX, AnnotationBar } from "./annotation-bar";
import { type TimelineAnnotationPin } from "./annotation-bar-pin";
import {
  carryScrollSpeed,
  type Held,
  heldLanes,
  holdStep,
} from "./annotation-hold";
import { packLanes } from "./annotation-lanes";

import type { AnnotationKind } from "../../../../bindings/AnnotationKind";

// Every annotation across the time it shows, over the beats, each in its
// lane; the lanes are the stacking order, the top one drawn in front, and
// magnetic: each bar rises as high as that order lets it
// (`annotation-lanes.ts`). Dragged, a bar follows the pointer in time and
// up and down, between just above the top lane and just below the bottom
// one, and the others glide to where they will sit when it is let go: into
// the lane under it where it fits, or a new lane opened there; the slots
// hold steady near their edges (`annotation-hold.ts`). Kept to its lane
// with Shift, or held by an end, a bar moved onto another in its lane takes
// a new lane of its own, above or below as the pointer is, and goes back
// once it is clear; the others rise into any room it leaves. With ⌘ held,
// ends snap to what `snap` offers, the strip's one list of edges
// (`strip-snap.ts`), which leaves out the bar's own. The lanes grow to
// `MAX_SHOWN_LANES`; past that they scroll inside that height, and the rest
// of the strip stays put. A bar carried near their top or bottom scrolls
// them on, staying under the pointer.

/** The most lanes shown at once; past them the lanes scroll. */
const MAX_SHOWN_LANES = 4;

/** How far below the lanes a snapped end's guide reaches, down through the
 * beats to the event it holds to, in px. */
const GUIDE_BELOW_PX = 72;

export type TimelineAnnotation = Span & {
  id: string;
  kind: AnnotationKind;
  /** Its lane, 0 at the top, which is drawn in front. */
  lane: number;
  /** Its pin, where it follows the content under it. */
  pin?: TimelineAnnotationPin;
};

export type AnnotationLaneProps = {
  annotations: TimelineAnnotation[];
  /** Told every annotation's lane, by id, after a drag moved any. */
  onRestack: (lanes: Record<string, number>) => void;
  onRetime: (id: string, start: number, end: number) => void;
  onSelect: (id: string) => void;
  scale: TimelineScale;
  /** The strip's width, in px. */
  width: number;
  selected?: string;
  /** Where a dragged end settles while ⌘ is held, given the annotation it
   * belongs to, whose own edges are left out; without it, ends go where
   * they are let go. */
  snap?: SnapFor;
};

export function AnnotationLane({
  annotations,
  onRestack,
  onRetime,
  onSelect,
  scale,
  selected,
  snap,
  width,
}: AnnotationLaneProps) {
  const reducedMotion = useReducedMotion() ?? false;
  const [held, setHeld] = useState<Held | null>(null);
  // Lanes are about what shows at once as it plays, so they are packed on
  // where the bars are drawn, which a reordered recording no longer keeps in
  // recorded order.
  const drawnSpan = (span: Span) => ({
    end: scale.toX(span.end, true),
    start: scale.toX(span.start),
  });
  const drawn = annotations.map((each) => ({ ...each, ...drawnSpan(each) }));
  // The lanes risen, as the strip shows them with nothing held; held, where
  // every bar goes if it is let go now.
  const settled = packLanes(drawn);
  const lanes = held ? heldLanes(drawn, held) : settled;
  const laneOf = (annotation: TimelineAnnotation) => lanes[annotation.id];
  const shownCount = Math.max(
    1,
    ...annotations.map((annotation) => laneOf(annotation) + 1),
    ...(held?.liftedTop == null
      ? []
      : [Math.ceil(held.liftedTop / ANNOTATION_BAR_PX + 0.5)]),
  );
  // Only the rows the bars use: a chosen bar's handles, a little taller
  // than its row, show in the scrolling box's margin.
  const contentHeight = shownCount * ANNOTATION_BAR_PX;
  const height = Math.min(contentHeight, MAX_SHOWN_LANES * ANNOTATION_BAR_PX);
  const [viewport, setViewport] = useState<HTMLElement | null>(null);
  const [scrollTop, setScrollTop] = useState(0);
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
  useEffect(() => {
    if (!viewport || !isCarried) return;
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
  }, [height, isCarried, viewport]);
  // Drawn over the lanes rather than in them, so it is not cut off where
  // they scroll: from the held bar, as far as it shows, down past them.
  const guide =
    held?.guide == null
      ? null
      : {
          left: scale.toX(held.guide) * width,
          top: Math.min(
            Math.max(
              (held.liftedTop ?? lanes[held.id] * ANNOTATION_BAR_PX) -
                scrollTop,
              0,
            ),
            height,
          ),
        };

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
        onInitialized={setViewport}
        rootClassName="pointer-events-none absolute -inset-x-2 -inset-y-2.5 h-auto w-auto"
      >
        <div
          className="pointer-events-auto relative"
          style={{ height: contentHeight }}
        >
          {annotations.map((annotation) => {
            const liftedTop =
              annotation.id === held?.id ? held.liftedTop : null;
            return (
              <AnnotationBar
                isLifted={liftedTop !== null}
                isSelected={baseOf(annotation.id) === selected}
                key={annotation.id}
                kind={annotation.kind}
                onHold={(hold) => {
                  if (hold) {
                    pointerRef.current = hold.y - (viewport?.scrollTop ?? 0);
                    setHeld((current) =>
                      holdStep({
                        annotations: drawn,
                        current,
                        hold: { ...hold, span: drawnSpan(hold.span) },
                        id: annotation.id,
                        shown: settled,
                      }),
                    );
                    return;
                  }
                  // Every move renders the lane again, so the bar is let go
                  // with this render's handler and its lanes.
                  setHeld(null);
                  if (annotations.some(({ id, lane }) => lanes[id] !== lane))
                    onRestack(lanes);
                }}
                onRetime={(start, end) => {
                  onRetime(annotation.id, start, end);
                }}
                onSelect={() => {
                  onSelect(annotation.id);
                }}
                pin={annotation.pin}
                scale={scale}
                snap={snap?.(baseOf(annotation.id))}
                span={annotation}
                top={liftedTop ?? laneOf(annotation) * ANNOTATION_BAR_PX}
                width={width}
              />
            );
          })}
        </div>
      </ScrollArea>
      {guide === null ? null : (
        <SnapGuide
          style={{
            height: height - guide.top + GUIDE_BELOW_PX,
            insetInlineStart: guide.left,
            top: guide.top,
          }}
        />
      )}
    </motion.div>
  );
}
