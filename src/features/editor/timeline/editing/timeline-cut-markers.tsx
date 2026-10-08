// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { UnfoldHorizontal } from "lucide-react";
import { useNumberFormatter } from "react-aria";

import { IconButton } from "../../../../components/base/button/icon-button";
import { NativeTooltipTrigger } from "../../../../components/shared/native-tooltip/native-tooltip-trigger";
import { t } from "../../../../i18n/i18n";
import { formatDuration } from "../../duration";
import { TimelineViewportState } from "../timeline-viewport";

import {
  RecordingTimelineCut,
  recordingTimelineCuts,
} from "./recording-timeline-cuts";
import { recordingTimelineRetainedDuration } from "./recording-timeline-edit";
import { TimelineBladeController } from "./timeline-blade";

type Anchor = "end" | "join" | "start";
const anchor = (cut: RecordingTimelineCut): Anchor =>
  cut.segmentId === null
    ? "start"
    : cut.followingSegmentId === null
      ? "end"
      : "join";
const MARKER_ANCHOR: Record<Anchor, string> = {
  end: "-translate-x-full",
  join: "-translate-x-1/2",
  start: "",
};

/**
 * A marker over the ruler at every place part of the recording was cut out,
 * which puts that part back. A cut is the timeline's and not any one lane's,
 * so it is marked once along the ruler, as a retime is, rather than in every
 * lane the join runs through. A join's marker is centred on it; a marker at
 * either end of the recording sits wholly inside the timeline.
 */
export function TimelineCutMarkers({
  blade,
  durationMs,
  viewport,
}: {
  blade: TimelineBladeController;
  /** The cut timeline's length, which the times are given on. */
  durationMs: number;
  viewport: TimelineViewportState;
}) {
  const seconds = useNumberFormatter({
    maximumFractionDigits: 1,
    style: "unit",
    unit: "second",
    unitDisplay: "short",
  });
  const cuts = recordingTimelineCuts(blade.edit);
  if (cuts.length === 0) return null;
  // The cut-out parts are measured on the whole recording, which the kept
  // share of it stretches to the timeline's length.
  const retained = recordingTimelineRetainedDuration(blade.edit);
  const sourceMs = retained > 0 ? durationMs / retained : 0;
  return (
    <div className="pointer-events-none absolute inset-0">
      {cuts.map((cut) => {
        const left = (cut.outputPosition - viewport.panOffset) * viewport.zoom;
        if (left < 0 || left > 1) return null;
        const duration = seconds.format(
          ((cut.sourceEnd - cut.sourceStart) * sourceMs) / 1_000,
        );
        const time = formatDuration(cut.outputPosition * durationMs);
        return (
          // The lower half of the ruler, which the ticks leave free while any
          // cut is marked, as they do for a retime strip. A retime strip
          // meeting the cut insets itself to clear the marker and a gap.
          <div
            className={`absolute bottom-0 flex ${MARKER_ANCHOR[anchor(cut)]}`}
            key={cut.sourceStart}
            style={{ left: `${(left * 100).toString()}%` }}
          >
            <NativeTooltipTrigger
              tooltip={t("editor-timeline-restore-cut", { duration })}
            >
              <IconButton
                aria-label={t("editor-timeline-restore-cut-at", {
                  duration,
                  time,
                })}
                className="pointer-events-auto"
                onPress={() => {
                  blade.restoreCut(cut);
                }}
                size="compact"
              >
                <UnfoldHorizontal aria-hidden />
              </IconButton>
            </NativeTooltipTrigger>
          </div>
        );
      })}
    </div>
  );
}
