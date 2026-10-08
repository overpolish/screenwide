// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { recordingTimelineCuts } from "../editing/recording-timeline-cuts";
import { RecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { speedMarkedSegments } from "../editing/timeline-speed-markers";
import { TimelineViewportState } from "../timeline-viewport";

/**
 * Whether a pin at an output position would run into what the ruler draws in
 * its lower half: a retime strip, which runs the length of its clip, or a cut
 * marker, centred on its join or, at either end of the timeline, lying
 * wholly inside it. Such a pin keeps to the upper half with the ticks; every
 * other pin stands the ruler's full height.
 *
 * A pin and a centred marker meet when their centres are closer than half of
 * both widths together. A marker at an end reaches its whole width in, so a
 * pin there keeps both widths clear: half a pin more than it strictly needs.
 * The widths are tokens, so the probe the caller renders is sized from them
 * and measured, and the root is measured for the pixels the output positions
 * spread over.
 */
export function useRulerCrowding(
  edit: RecordingTimelineEdit,
  viewport: TimelineViewportState,
  isMounted: boolean,
) {
  const rootRef = useRef<HTMLDivElement>(null);
  const probeRef = useRef<HTMLSpanElement>(null);
  const [measure, setMeasure] = useState({ clearance: 0, width: 0 });

  // A ResizeObserver reports each element once as it starts watching, so the
  // first measure arrives without reading layout during the effect.
  useEffect(() => {
    const root = rootRef.current;
    const probe = probeRef.current;
    if (!isMounted || !root || !probe) return;
    const observer = new ResizeObserver(() => {
      setMeasure({ clearance: probe.offsetWidth, width: root.clientWidth });
    });
    observer.observe(root);
    observer.observe(probe);
    return () => {
      observer.disconnect();
    };
  }, [isMounted]);

  const strips = speedMarkedSegments(edit);
  const cuts = recordingTimelineCuts(edit);
  const isCrowded = (output: number) =>
    strips.some(
      (strip) => output >= strip.outputStart && output <= strip.outputEnd,
    ) ||
    cuts.some((cut) => {
      const apart = Math.abs(
        (output - cut.outputPosition) * viewport.zoom * measure.width,
      );
      const atEdge = cut.segmentId === null || cut.followingSegmentId === null;
      return apart < (atEdge ? 2 * measure.clearance : measure.clearance);
    });

  return { isCrowded, probeRef, rootRef };
}
