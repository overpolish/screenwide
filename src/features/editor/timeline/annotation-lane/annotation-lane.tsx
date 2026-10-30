// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { motion, useReducedMotion } from "motion/react";
import { useState } from "react";

import { motionDurations, motionEasings } from "../../../../lib/motion";
import { type Span, type TimelineScale } from "../timeline-scale";

import { ANNOTATION_BAR_PX, AnnotationBar } from "./annotation-bar";
import { type Held, heldLanes, holdStep } from "./annotation-hold";
import { packLanes } from "./annotation-lanes";
import { type Snap } from "./use-annotation-bar-drag";

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
// ends snap to the starts and ends of the other annotations, and to
// whatever else `snap` offers, whichever is nearest.

/** How near, in px, an end comes to another annotation's end to snap. */
const SNAP_PX = 8;

/** The lanes' least height, in px: one lane, with room for a chosen bar's
 * handles. */
const MIN_HEIGHT_PX = 24;

export type TimelineAnnotation = Span & {
  id: string;
  kind: AnnotationKind;
  /** Its lane, 0 at the top, which is drawn in front. */
  lane: number;
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
  /** Where else a dragged end settles while ⌘ is held, besides the other
   * annotations' starts and ends, such as events in the recording. */
  snap?: Snap;
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
  // The lanes risen, as the strip shows them with nothing held; held, where
  // every bar goes if it is let go now.
  const settled = packLanes(annotations);
  const lanes = held ? heldLanes(annotations, held) : settled;
  const laneOf = (annotation: TimelineAnnotation) => lanes[annotation.id];
  const shownCount = Math.max(
    1,
    ...annotations.map((annotation) => laneOf(annotation) + 1),
    ...(held?.liftedTop == null
      ? []
      : [Math.ceil(held.liftedTop / ANNOTATION_BAR_PX + 0.5)]),
  );
  // The nearest of the other annotations' starts and ends in reach, or what
  // `snap` offers, if that is nearer.
  const snapFor =
    (id: string): Snap =>
    (seconds) => {
      const offset = (to: number) =>
        Math.abs(scale.toX(to) - scale.toX(seconds)) * width;
      const given = snap?.(seconds);
      let best = given?.isSnapped ? given : { isSnapped: false, seconds };
      let nearest = best.isSnapped ? offset(best.seconds) : SNAP_PX;
      for (const other of annotations) {
        if (other.id === id) continue;
        for (const edge of [other.start, other.end]) {
          if (offset(edge) > nearest) continue;
          best = { isSnapped: true, seconds: edge };
          nearest = offset(edge);
        }
      }
      return best;
    };

  return (
    <motion.div
      animate={{
        height: Math.max(MIN_HEIGHT_PX, shownCount * ANNOTATION_BAR_PX),
      }}
      className="relative"
      dir="ltr"
      initial={false}
      transition={{
        duration: reducedMotion ? 0 : motionDurations.travel,
        ease: motionEasings.out,
      }}
    >
      {annotations.map((annotation) => {
        const liftedTop = annotation.id === held?.id ? held.liftedTop : null;
        return (
          <AnnotationBar
            isLifted={liftedTop !== null}
            isSelected={annotation.id === selected}
            key={annotation.id}
            kind={annotation.kind}
            onHold={(hold) => {
              if (hold) {
                setHeld((current) =>
                  holdStep({
                    annotations,
                    current,
                    hold,
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
            scale={scale}
            snap={snapFor(annotation.id)}
            span={annotation}
            top={liftedTop ?? laneOf(annotation) * ANNOTATION_BAR_PX}
            width={width}
          />
        );
      })}
    </motion.div>
  );
}
