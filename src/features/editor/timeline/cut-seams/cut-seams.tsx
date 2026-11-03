// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Undo2 } from "lucide-react";
import { useNumberFormatter } from "react-aria";

import { t } from "../../../../i18n/i18n";
import { TimelineMark } from "../timeline-mark";
import { at, type Span, type TimelineScale } from "../timeline-scale";

// Where a cut closed up, a seam down the beats: pressed, it puts the cut
// back. Cuts that touch or overlap close up at one point, so they are one
// seam, which puts all of them back.

export type TimelineCut = Span & { id: string };

/** Cuts that touch or overlap, as runs, in order. */
function runsOf(cuts: TimelineCut[]) {
  const runs: (Span & { ids: string[]; seconds: number })[] = [];
  const sorted = [...cuts].sort((first, second) => first.start - second.start);
  for (const cut of sorted) {
    const last = runs[runs.length - 1] as (typeof runs)[number] | undefined;
    if (last && cut.start <= last.end + 0.001) {
      last.seconds += Math.max(0, cut.end - Math.max(cut.start, last.end));
      last.end = Math.max(last.end, cut.end);
      last.ids.push(cut.id);
    } else
      runs.push({
        end: cut.end,
        ids: [cut.id],
        seconds: cut.end - cut.start,
        start: cut.start,
      });
  }
  return runs;
}

export function CutSeams({
  cuts,
  onRestore,
  scale,
}: {
  cuts: TimelineCut[];
  /** Told the cuts a seam puts back. */
  onRestore: (ids: string[]) => void;
  scale: TimelineScale;
}) {
  const seconds = useNumberFormatter({
    maximumFractionDigits: 1,
    style: "unit",
    unit: "second",
    unitDisplay: "short",
  });
  return runsOf(cuts).map((run) => (
    <TimelineMark
      end="top"
      glyph={<Undo2 aria-hidden />}
      isDashed
      key={run.ids[0]}
      label={t("editor-timeline-restore-cut", {
        duration: seconds.format(run.seconds),
      })}
      onPress={() => {
        onRestore(run.ids);
      }}
      style={at(scale, run.start)}
    />
  ));
}
