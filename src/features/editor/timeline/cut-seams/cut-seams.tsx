// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Undo2 } from "lucide-react";
import { useNumberFormatter } from "react-aria";

import { t } from "../../../../i18n/i18n";
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
    <button
      aria-label={t("editor-timeline-restore-cut", {
        duration: seconds.format(run.seconds),
      })}
      className="group absolute inset-y-0 z-10 flex w-4 -translate-x-1/2 cursor-default items-start justify-center outline-none"
      dir="ltr"
      key={run.ids[0]}
      onClick={() => {
        onRestore(run.ids);
      }}
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
      style={at(scale, run.start)}
      type="button"
    >
      <span className="absolute inset-y-0 w-px border-s border-dashed border-fg-secondary" />
      <span className="relative -mt-1 flex size-4 items-center justify-center rounded-full bg-fg text-surface group-focus-visible:outline-2 group-focus-visible:outline-offset-1 group-focus-visible:outline-focus">
        <Undo2 aria-hidden className="size-icon-xs" />
      </span>
    </button>
  ));
}
