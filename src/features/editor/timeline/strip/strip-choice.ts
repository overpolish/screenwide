// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Span } from "../timeline-scale";

import { playedSpan } from "./strip-lane-time";
import { baseOf } from "./strip-part";
import { selectionSpan, type StripSelection } from "./strip-selection";
import { type TimelineStripProps } from "./timeline-strip-props";

// What is chosen on the strip, and choosing: pressing a part chooses it and
// leaves the playhead where it is (#83). The islands act on what is chosen;
// the picture shows the playhead's moment. Every part is found where it
// plays, in played seconds: the lanes' are there already, the beats' and
// lulls' are read from the recorded seconds they keep.

export function stripChoice(props: TimelineStripProps) {
  const { map, selection } = props;
  const onLane = (each: Span & { id: string }) => ({
    id: each.id,
    span: { end: each.end, start: each.start },
  });
  /** A part of the footage, kept in recorded seconds, where it plays. */
  const onFootage = (each: { id: string; span: Span }) => ({
    id: each.id,
    span: playedSpan(map, each.span),
  });
  const parts = {
    annotation: props.annotations.map(onLane),
    audioClip: (props.audio?.clips ?? []).map(onLane),
    beat: props.beats.map(onFootage),
    lull: props.lulls.map(onFootage),
    scene: props.scenes.map(onLane),
  };
  const choose = (next: StripSelection | null) => {
    props.onSelect(next);
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
