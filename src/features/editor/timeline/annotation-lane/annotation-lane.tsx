// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { BarStack } from "../bar-stack/bar-stack";
import { type IncomingBar } from "../bar-stack/use-incoming-bar";
import { type SnapFor } from "../strip/strip-snap";
import { type Span, type TimelineScale } from "../timeline-scale";

import { AnnotationBar } from "./annotation-bar";
import { type TimelineAnnotationPin } from "./annotation-bar-pin";

import type { AnnotationKind } from "../../../../bindings/AnnotationKind";

// Every annotation across the time it shows, over the beats, in a stack of
// lanes (`bar-stack.tsx`): the lanes are the stacking order, the top one
// drawn in front.

export type TimelineAnnotation = Span & {
  id: string;
  kind: AnnotationKind;
  /** Its lane, 0 at the top, which is drawn in front. */
  lane: number;
  /** Where its annotation travels between its places in the scenes. */
  glides?: Span[];
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
  /** A picture carried in, held as a bar of its own (`BarStack`). */
  incoming?: IncomingBar<TimelineAnnotation> | null;
  /** Told every bar's lane if the carried picture is let go now, or null. */
  onIncomingLanes?: (lanes: Record<string, number> | null) => void;
  selected?: string;
  /** Where a dragged end settles while ⌘ is held, given its annotation,
   * whose own edges are left out. */
  snap?: SnapFor;
};

export function AnnotationLane({
  annotations,
  onRetime,
  ...stack
}: AnnotationLaneProps) {
  return (
    <BarStack
      {...stack}
      bars={annotations}
      guideTo="below"
      onRetime={(id, { end, start }) => {
        onRetime(id, start, end);
      }}
      renderBar={(annotation, placement) => (
        <AnnotationBar
          {...placement}
          glides={annotation.glides}
          kind={annotation.kind}
          pin={annotation.pin}
        />
      )}
    />
  );
}
