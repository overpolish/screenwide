// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type BeatTrim } from "../beat-clip/beat-trim";
import { type TimelineCaption } from "../caption-lane/caption-line";
import { type TimelineCut } from "../cut-seams/cut-seams";
import { type Span, type TimeMap } from "../timeline-scale";

import { type StripBeat } from "./strip-beat-row";

// A beat's trim, as the strip reads it: what plays of it is what its cuts
// leave, a trim at either end being a cut there; and said aloud, its ends
// settle between words, so a trim never takes half of one.

/** Close enough, in seconds, for a cut to meet a beat's edge. */
const TOUCH_S = 1e-6;

/** The part of `span` that plays: its ends brought in past the cuts that
 * reach them. */
export function keptOf(span: Span, cuts: TimelineCut[]): Span {
  const sorted = [...cuts].sort((first, second) => first.start - second.start);
  let { end, start } = span;
  for (const cut of sorted)
    if (cut.start <= start + TOUCH_S && cut.end > start)
      start = Math.min(cut.end, end);
  for (const cut of sorted.reverse())
    if (cut.end >= end - TOUCH_S && cut.start < end)
      end = Math.max(cut.start, start);
  return { end, start };
}

/** How near, in px, a dragged end comes to a word's start to settle on it:
 * small, as words sit only a few px apart when the strip shows much. */
const SNAP_PX = 4;

/** The moments words start at in `span`, and its edges: where a trim of a
 * line said there may settle. */
const wordStarts = (span: Span, captions: TimelineCaption[] | null) => [
  span.start,
  span.end,
  ...(captions ?? []).flatMap(({ words }) =>
    words.map(({ at }) => at).filter((at) => at > span.start && at < span.end),
  ),
];

export function beatTrimOf({
  beat,
  captions,
  cuts,
  map,
  onTrim,
  width,
}: {
  beat: StripBeat;
  captions: TimelineCaption[] | null;
  cuts: TimelineCut[];
  map: TimeMap;
  onTrim: (id: string, edge: "end" | "start", at: number) => void;
  /** The track's width, in px. */
  width: number;
}): BeatTrim {
  const starts =
    beat.beat.kind === "speech" ? wordStarts(beat.span, captions) : null;
  const pxPerSecond = width / Math.max(map.duration, 1e-6);
  return {
    kept: keptOf(beat.span, cuts),
    onTrim: (edge, at) => {
      onTrim(beat.id, edge, at);
    },
    pxPerSecond,
    // A word's start in reach catches the end, so a trim can stop clean
    // between words; further off, the end goes where it is dragged.
    snap: starts
      ? (seconds) => {
          const nearest = starts.reduce((best, at) =>
            Math.abs(at - seconds) < Math.abs(best - seconds) ? at : best,
          );
          return Math.abs(nearest - seconds) * pxPerSecond <= SNAP_PX
            ? nearest
            : seconds;
        }
      : undefined,
  };
}
