// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { BarStack } from "../bar-stack/bar-stack";
import { type BarPart } from "../bar-stack/use-stack-bar-drag";
import { baseOf } from "../strip/strip-part";
import { type SnapFor } from "../strip/strip-snap";
import { type Span, type TimelineScale } from "../timeline-scale";

import { AudioBar, type TimelineAudioClip } from "./audio-bar";

// The sound effects in a stack of lanes, drawn as the annotations are
// (`bar-stack.tsx`): each a line across the time it plays, carried in time
// and up and down through lanes that are an order of their own, its ends
// pulled to trim it. They overlap freely, so the lanes follow the pointer.
// The songs are on a rail of their own under them (`song-rail.tsx`).

export type { TimelineAudioClip };

/** The songs and sound effects as the strip shows them, and what it tells
 * of each change made to them. */
export type StripAudio = {
  clips: TimelineAudioClip[];
  /** Told every clip's lane, by id, after a drag moved any. */
  onRestack: (lanes: Record<string, number>) => void;
  /** Told a clip moved whole or an end of it pulled, in recorded seconds. */
  onRetime: (id: string, span: Span, part: BarPart) => void;
};

export type AudioStackProps = Omit<StripAudio, "clips"> & {
  /** The sound effects. */
  clips: TimelineAudioClip[];
  onSelect: (id: string) => void;
  scale: TimelineScale;
  /** The strip's width, in px. */
  width: number;
  selected?: string;
  snap?: SnapFor;
};

export function AudioStack({ onSelect, ...stack }: AudioStackProps) {
  return (
    <BarStack
      {...stack}
      bars={stack.clips}
      guideTo="above"
      // A clip drawn either side of a jump is chosen whole.
      onSelect={(id) => {
        onSelect(baseOf(id));
      }}
      renderBar={(clip, placement) => <AudioBar {...placement} clip={clip} />}
    />
  );
}
