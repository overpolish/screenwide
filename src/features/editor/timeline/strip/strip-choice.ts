// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { baseOf } from "./strip-part";
import {
  isPlayheadOutside,
  selectionSpan,
  type StripSelection,
} from "./strip-selection";
import { type TimelineStripProps } from "./timeline-strip-props";

// What is chosen on the strip, and choosing: pressing a part chooses it
// and brings the playhead to its start where the playhead stands outside
// it, so the picture shows what was chosen.

export function stripChoice(props: TimelineStripProps) {
  const { map, selection } = props;
  const parts = {
    annotation: props.annotations.map((each) => ({ id: each.id, span: each })),
    // Laid on the timeline as it plays, each is found where it plays.
    audioClip: (props.audio?.clips ?? []).map((each) => ({
      id: each.id,
      span: {
        end: map.toRecorded(each.end),
        start: map.toRecorded(each.start),
      },
    })),
    beat: props.beats,
    lull: props.lulls,
    scene: props.scenes.map((each) => ({ id: each.id, span: each })),
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
    const at = map.toRecorded(props.playhead.ratio() * map.duration);
    if (isPlayheadOutside(at, span)) props.onSeek(span.landsAt ?? span.start);
  };
  const chosenId = (kind: "annotation" | "audioClip" | "scene") =>
    selection?.kind === kind ? baseOf(selection.id) : undefined;
  return { choose, chosen: selectionSpan(selection, parts), chosenId };
}
