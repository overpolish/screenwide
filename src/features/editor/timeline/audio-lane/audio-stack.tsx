// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { BarStack } from "../bar-stack/bar-stack";
import { type BarPart } from "../bar-stack/use-stack-bar-drag";
import { baseOf } from "../strip/strip-part";
import { type SnapFor } from "../strip/strip-snap";
import { type Span, type TimelineScale } from "../timeline-scale";

import { AudioBar, type TimelineAudioClip } from "./audio-bar";

// The songs and sound effects under the beats, in a stack of lanes as the
// annotations over them are (`bar-stack.tsx`). Sound effects go anywhere;
// a song stops against the songs either side, or jumps past one pushed
// beyond its middle (`movedClip`), and with no key held snaps to their ends
// within reach, so songs are easily laid end to end. They are
// laid on the timeline as it plays, so a cut or a speed change under them
// never moves them.

export type { TimelineAudioClip };

/** How near, in px, a song's edge comes to another song's before it snaps
 * to it, with no key held, so songs are easily laid end to end. */
const SONG_SNAP_PX = 8;

/** The songs and sound effects as the strip shows them, and what it tells
 * of each change made to them. */
export type StripAudio = {
  clips: TimelineAudioClip[];
  /** Told every clip's lane, by id, after a drag moved any. */
  onRestack: (lanes: Record<string, number>) => void;
  /** Told a clip moved whole or an end of it pulled, in recorded seconds. */
  onRetime: (id: string, span: Span, part: BarPart) => void;
};

export type AudioStackProps = StripAudio & {
  onSelect: (id: string) => void;
  scale: TimelineScale;
  /** The strip's width, in px. */
  width: number;
  selected?: string;
  snap?: SnapFor;
};

export function AudioStack({
  clips,
  onRetime,
  onSelect,
  ...stack
}: AudioStackProps) {
  const { scale, width } = stack;
  /** The other songs' edge nearest `seconds` within snapping reach of it,
   * if any. */
  const songEdgeNear = (id: string, seconds: number) => {
    const x = scale.toX(seconds) * width;
    let nearest: { distance: number; seconds: number } | null = null;
    for (const other of clips) {
      if (other.kind !== "music" || baseOf(other.id) === baseOf(id)) continue;
      for (const edge of [other.start, other.end]) {
        const distance = Math.abs(scale.toX(edge) * width - x);
        if (distance <= SONG_SNAP_PX && distance < (nearest?.distance ?? 1 / 0))
          nearest = { distance, seconds: edge };
      }
    }
    return nearest?.seconds ?? null;
  };
  /** A song's span with the edge that moved snapped to a neighbour's: the
   * start or the end where one end moved, either where the whole moved. */
  const snapped = (id: string, { end, start }: Span, part: BarPart) => {
    const clip = clips.find((each) => each.id === id);
    if (clip?.kind !== "music") return { end, start };
    const head = part === "end" ? null : songEdgeNear(id, start);
    const tail =
      part === "start" || head !== null ? null : songEdgeNear(id, end);
    if (part === "body") {
      const shift = (head ?? start) - start + ((tail ?? end) - end);
      return { end: end + shift, start: start + shift };
    }
    return { end: tail ?? end, start: head ?? start };
  };
  return (
    <BarStack
      {...stack}
      bars={clips}
      // Songs stop against and jump over each other as they are moved, so
      // the lanes follow where each lands, not the pointer.
      followsPlacement
      guideTo="above"
      // Sound is not drawn, so the lanes have no order to choose: they
      // only keep clips that overlap apart.
      movesInTimeOnly
      onRetime={(id, span, part) => {
        onRetime(id, snapped(id, span, part), part);
      }}
      // A clip drawn either side of a jump is chosen whole.
      onSelect={(id) => {
        onSelect(baseOf(id));
      }}
      renderBar={(clip, placement) => <AudioBar {...placement} clip={clip} />}
    />
  );
}
