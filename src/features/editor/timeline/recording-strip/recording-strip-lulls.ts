// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type TimelineCut } from "../cut-seams/cut-seams";
import { type RecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { type StripLull } from "../strip/strip-beat-row";
import { keptOf } from "../strip/strip-beat-trim";
import { type Span } from "../timeline-scale";

import { type RecordingLull } from "./recording-beats";
import { atRate } from "./recording-strip-source";

import type { LullPlay } from "../../island/lull/lull-island";

// How the lulls play, read from the speed the timeline edit plays them at,
// which is all the edit keeps of them. Squeezed, every lull is hurried by
// the same share of what it can give: up to four times as fast, or twice
// where the picture moves by itself, since motion played faster cannot be
// followed. That share is the one most lulls play at; a lull at its own
// speed is retimed, and one as recorded while the rest are squeezed is
// kept. With nothing squeezed, every lull is squeezed with the rest, by
// nothing: the edit cannot tell a kept lull from one that waits to be.

/** How long, in seconds, a squeezed lull can still play and be dead air. */
const DEAD_AIR_S = 3;
/** How near two shares of give are to be one. */
const SAME_SHARE = 2e-3;

/** A lull as the strip draws it, with what changing it needs. */
type PlayedLull = StripLull & {
  /** As fast as it can be squeezed. */
  fastest: number;
  /** The whole lull as found, which a trim can bring back to. */
  found: Span;
};

export type RecordingStripLulls = {
  lulls: PlayedLull[];
  maxSqueeze: number;
  /** The share of their give the squeezed lulls play with, from 0 to 1. */
  share: number;
  /** How many seconds the lulls are squeezed by. */
  squeeze: number;
};

/** The speed a lull squeezed by `share` of its give plays at. */
const squeezedRate = (share: number, fastest: number) =>
  1 / (1 - share * (1 - 1 / fastest));

const giveOf = ({ fastest, span }: PlayedLull) =>
  (span.end - span.start) * (1 - 1 / fastest);

/** The share most of the hurried lulls play with, or 0 with none hurried. */
function commonShare(shares: number[]) {
  const counts = new Map<number, number>();
  for (const share of shares) {
    const key = Math.round(share / SAME_SHARE) * SAME_SHARE;
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  let best = 0;
  let most = 0;
  for (const [share, count] of counts)
    if (count > most || (count === most && share > best)) {
      best = share;
      most = count;
    }
  return best;
}

export function recordingStripLulls({
  cuts,
  edit,
  found,
  recorded,
}: {
  cuts: TimelineCut[];
  edit: RecordingTimelineEdit;
  found: RecordingLull[];
  recorded: number;
}): RecordingStripLulls {
  const rateAt = (seconds: number) =>
    edit.segments.find(
      (segment) =>
        seconds / recorded >= segment.sourceStart &&
        seconds / recorded < segment.sourceEnd,
    )?.playbackRate ?? 1;
  const read = found.flatMap((lull) => {
    const span = keptOf(lull.span, cuts);
    if (span.end - span.start <= 0) return [];
    const fastest = lull.moving ? 2 : 4;
    const rate = rateAt(span.start);
    const share =
      rate > 1 && rate <= fastest + 1e-6
        ? (1 - 1 / rate) / (1 - 1 / fastest)
        : null;
    return [{ fastest, lull, rate, share, span }];
  });
  const share = commonShare(read.flatMap((each) => each.share ?? []));
  const lulls = read.map(({ fastest, lull, rate, share: own, span }) => {
    const play: LullPlay =
      rate === 1
        ? { kind: share > 0 ? "keep" : "squeeze" }
        : own !== null && Math.abs(own - share) < SAME_SHARE
          ? { kind: "squeeze" }
          : { kind: "retimed", speed: rate };
    const plays = (span.end - span.start) / rate;
    return {
      fastest,
      found: lull.span,
      id: lull.id,
      isDeadAir: play.kind === "squeeze" && plays >= DEAD_AIR_S,
      moving: lull.moving,
      play,
      plays,
      span,
    };
  });
  const maxSqueeze = lulls
    .filter(({ play }) => play.kind === "squeeze")
    .reduce((sum, lull) => sum + giveOf(lull), 0);
  return { lulls, maxSqueeze, share, squeeze: share * maxSqueeze };
}

/** `edit` playing `span`, in recorded seconds, at `rate`. */
const spanAt = (
  edit: RecordingTimelineEdit,
  span: Span,
  { rate, recorded }: { rate: number; recorded: number },
) =>
  atRate(edit, {
    end: span.end / recorded,
    rate,
    start: span.start / recorded,
  });

/** `edit` with the squeezed lulls hurried to give `squeeze` seconds. */
export function squeezedTo(
  edit: RecordingTimelineEdit,
  { lulls, maxSqueeze }: RecordingStripLulls,
  { recorded, squeeze }: { recorded: number; squeeze: number },
): RecordingTimelineEdit {
  const share =
    maxSqueeze > 0 ? Math.min(Math.max(squeeze / maxSqueeze, 0), 1) : 0;
  return lulls
    .filter(({ play }) => play.kind === "squeeze")
    .reduce(
      (current, lull) =>
        spanAt(current, lull.span, {
          rate: squeezedRate(share, lull.fastest),
          recorded,
        }),
      edit,
    );
}

/** `edit` playing the lull `id` as `play` says. */
export function lullPlayed(
  edit: RecordingTimelineEdit,
  { lulls, share }: RecordingStripLulls,
  { id, play, recorded }: { id: string; play: LullPlay; recorded: number },
): RecordingTimelineEdit {
  const lull = lulls.find((each) => each.id === id);
  if (!lull) return edit;
  const rate =
    play.kind === "keep"
      ? 1
      : play.kind === "retimed"
        ? play.speed
        : squeezedRate(share, lull.fastest);
  return spanAt(edit, lull.span, { rate, recorded });
}
