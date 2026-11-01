// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Snap } from "../annotation-lane/use-annotation-bar-drag";
import { type Span, type TimelineScale } from "../timeline-scale";

import { baseOf } from "./strip-part";

// With ⌘ (Ctrl on Windows) held, an end dragged on the strip settles on the
// nearest edge in reach, whatever is dragged: a scene drawn or retimed, an
// annotation's bar moved or retimed. Every drag reads the one list of edges
// (`stripSnapTargets`): the beats', the lulls', the scenes', the
// annotations', the cuts' and the playhead, leaving out the dragged item's
// own, which move with it. Each item comes as the strip draws it, in parts
// either side of a jump, and each end is placed where it is drawn, as the
// end of what stops there, so an end settles where the eye sees it. A beat's
// trim is not among these drags: it cuts what plays, which moves the scenes
// and annotations, timed as they play, along with it, so their edges would
// not stay where it settled; its ends settle between words instead
// (`strip-beat-trim.ts`).

/** How close, in px, an end comes to an edge to snap. */
const SNAP_PX = 8;

/** A span whose edges a dragged end may settle on, with the annotation or
 * scene it is part of, if any, whose own drags leave it out. A moment, such
 * as the playhead, is a span with no length. */
export type SnapTarget = Span & { owner?: string };

/** Where a dragged end settles while ⌘ is held, given the annotation or
 * scene being dragged, if any, whose own edges are left out. */
export type SnapFor = (except: string | null) => Snap;

/** Every edge a dragged end may settle on: the parts of the annotations
 * and scenes, by the item each is part of, the beats, lulls and cuts, and
 * the playhead, a recorded moment, where it is known. */
export function stripSnapTargets({
  annotations = [],
  beats = [],
  cuts = [],
  lulls = [],
  playheadAt = null,
  scenes = [],
}: {
  annotations?: (Span & { id: string })[];
  beats?: { span: Span }[];
  cuts?: Span[];
  lulls?: { span: Span }[];
  playheadAt?: number | null;
  scenes?: (Span & { id: string })[];
}): SnapTarget[] {
  const owned = ({ end, id, start }: Span & { id: string }) => ({
    end,
    owner: baseOf(id),
    start,
  });
  return [
    ...annotations.map(owned),
    ...scenes.map(owned),
    ...beats.map(({ span }) => ({ end: span.end, start: span.start })),
    ...lulls.map(({ span }) => ({ end: span.end, start: span.start })),
    ...cuts.map(({ end, start }) => ({ end, start })),
    ...(playheadAt === null ? [] : [{ end: playheadAt, start: playheadAt }]),
  ];
}

/** Where an end dragged on the strip settles: the nearest edge of
 * `targets` in reach, measured where each is drawn, leaving out those of
 * the item `except`. */
export const edgeSnap =
  ({
    except = null,
    scale,
    targets,
    width,
  }: {
    scale: TimelineScale;
    targets: SnapTarget[];
    /** The track's width, in px. */
    width: number;
    except?: string | null;
  }): Snap =>
  (seconds) => {
    const x = scale.toX(seconds);
    let best = { isSnapped: false, seconds };
    let nearest = SNAP_PX;
    for (const target of targets) {
      if (except !== null && target.owner === except) continue;
      for (const [edge, isEnd] of [
        [target.start, false],
        [target.end, true],
      ] as const) {
        const offset = Math.abs(scale.toX(edge, isEnd) - x) * width;
        if (offset > nearest) continue;
        best = { isSnapped: true, seconds: edge };
        nearest = offset;
      }
    }
    return best;
  };

type SnapParts = Omit<Parameters<typeof stripSnapTargets>[0], "playheadAt">;

/** The strip's snap over `parts`, the playhead read with `playheadAt` as
 * an end is dragged, so it is where the playhead stands then. */
export const stripSnap =
  ({
    parts,
    playheadAt,
    scale,
    width,
  }: {
    parts: SnapParts;
    playheadAt: () => number;
    scale: TimelineScale;
    /** The track's width, in px. */
    width: number;
  }): SnapFor =>
  (except) =>
  (seconds) =>
    edgeSnap({
      except,
      scale,
      targets: stripSnapTargets({ ...parts, playheadAt: playheadAt() }),
      width,
    })(seconds);
