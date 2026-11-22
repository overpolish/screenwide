// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type TimelineCut } from "../cut-seams/cut-seams";
import { type RecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { type StripLull } from "../strip/strip-beat-row";
import { keptOf } from "../strip/strip-beat-trim";
import { type Span } from "../timeline-scale";

import { type RecordingLull } from "./recording-beats";
import { atRate, rateAt } from "./recording-strip-source";

import type { LullPlay } from "../../island/lull/lull-island";

// How the lulls play, read from the speed the timeline edit plays them at
// and from the lulls it keeps as recorded. Squeezed, every lull is hurried
// by the same share of what it can give: up to four times as fast, or twice
// where the picture moves by itself, since motion played faster cannot be
// followed. That share is the one most lulls play at; a lull at its own
// speed is retimed. A lull as recorded is kept where the edit keeps it,
// since with nothing squeezed every lull plays as recorded and its speed
// cannot tell; and, for an edit saved before it kept any, where the rest
// are squeezed.

/** How long, in seconds, a squeezed lull can still play and be dead air. */
const DEAD_AIR_S = 3;
/** How near two shares of give are to be one. */
const SAME_SHARE = 2e-3;
/** Closer than this to a lull's edge, as a share of the recording, is in
 * it. */
const EDGE = 1e-6;

/** Where the edit marks `found` as kept: its middle, as a share of the
 * recording, which stays inside it however its ends are trimmed. */
const keptMark = (found: Span, recorded: number) =>
  (found.start + found.end) / 2 / recorded;

const marks = (found: Span, recorded: number) => (at: number) =>
  at >= found.start / recorded - EDGE && at <= found.end / recorded + EDGE;

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
  const read = found.flatMap((lull) => {
    const span = keptOf(lull.span, cuts);
    if (span.end - span.start <= 0) return [];
    const fastest = lull.moving ? 2 : 4;
    const rate = rateAt(edit, span.start, recorded);
    const share =
      rate > 1 && rate <= fastest + 1e-6
        ? (1 - 1 / rate) / (1 - 1 / fastest)
        : null;
    return [{ fastest, lull, rate, share, span }];
  });
  const share = commonShare(read.flatMap((each) => each.share ?? []));
  const keptLulls = edit.keptLulls ?? [];
  const lulls = read.map(({ fastest, lull, rate, share: own, span }) => {
    const isKept = keptLulls.some(marks(lull.span, recorded));
    const play: LullPlay =
      rate === 1
        ? { kind: isKept || share > 0 ? "keep" : "squeeze" }
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

/** `edit` playing the lull `id` as `play` says, kept as recorded or not. */
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
  const isMarked = marks(lull.found, recorded);
  const others = (edit.keptLulls ?? []).filter((at) => !isMarked(at));
  const keptLulls =
    play.kind === "keep"
      ? [...others, keptMark(lull.found, recorded)].sort((a, b) => a - b)
      : others;
  const next = spanAt(edit, lull.span, { rate, recorded });
  if (keptLulls.length === 0) return withoutKeptLulls(next);
  const isSame =
    next === edit &&
    keptLulls.length === edit.keptLulls?.length &&
    keptLulls.every((at, index) => at === edit.keptLulls?.[index]);
  return isSame ? edit : { ...next, keptLulls };
}

/** `edit` keeping no lull, without the empty list a saved edit leaves
 * out. */
function withoutKeptLulls(edit: RecordingTimelineEdit): RecordingTimelineEdit {
  if (edit.keptLulls === undefined) return edit;
  const { keptLulls: _kept, ...rest } = edit;
  return rest;
}
