// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Span } from "../timeline-scale";

import { playedSpan } from "./strip-lane-time";
import { baseOf } from "./strip-part";
import {
  isPlayheadOutside,
  selectionSpan,
  type StripSelection,
} from "./strip-selection";
import { type TimelineStripProps } from "./timeline-strip-props";

// What is chosen on the strip, and choosing: pressing a part chooses it
// and brings the playhead to its start where the playhead stands outside
// it, so the picture shows what was chosen. Every part is found where it
// plays, in played seconds: the lanes' are there already, the beats' and
// lulls' are read from the recorded seconds they keep, and each says the
// recorded moment the playhead is brought to (`seek`).

export function stripChoice(props: TimelineStripProps) {
  const { map, selection } = props;
  /** A lane's part, in played seconds, brought to at its landing. */
  const onLane = (each: Span & { id: string; landsAt?: number }) => ({
    id: each.id,
    span: {
      end: each.end,
      seek: map.toRecorded(each.landsAt ?? each.start),
      start: each.start,
    },
  });
  /** A part of the footage, kept in recorded seconds, where it plays. */
  const onFootage = (each: { id: string; span: Span }) => ({
    id: each.id,
    span: { ...playedSpan(map, each.span), seek: each.span.start },
  });
  const parts = {
    annotation: props.annotations.map(onLane),
    audioClip: (props.audio?.clips ?? []).map(onLane),
    beat: props.beats.map(onFootage),
    lull: props.lulls.map(onFootage),
    scene: props.scenes.map(onLane),
  };
  /** Chooses `next`, bringing the playhead to where it lands where it
   * stands outside it; what Shift gathers leaves the playhead alone, as
   * does a drawn range, whose drag brought the playhead along its edge. */
  const choose = (next: StripSelection | null, isGathered = false) => {
    props.onSelect(next);
    if (isGathered || next === null) return;
    if (next.kind === "shortcuts" || next.kind === "range") return;
    const span = selectionSpan(next, parts);
    if (!span) return;
    if (isPlayheadOutside(props.playhead.ratio() * map.duration, span))
      props.onSeek(span.seek ?? map.toRecorded(span.start));
  };
  const chosenId = (kind: "annotation" | "audioClip" | "scene") =>
    selection?.kind === kind ? baseOf(selection.id) : undefined;
  /** What is chosen, in played seconds, which the view follows. */
  const chosen =
    selection?.kind === "range"
      ? playedSpan(map, selection)
      : selectionSpan(selection, parts);
  return { choose, chosen, chosenId };
}
