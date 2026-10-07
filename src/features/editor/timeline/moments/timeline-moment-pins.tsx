// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "react-aria-components";

import { NativeTooltipTrigger } from "../../../../components/shared/native-tooltip/native-tooltip-trigger";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";
import { formatDuration } from "../../duration";
import { SeekHandler } from "../timeline-seek";
import { TimelineViewportState } from "../timeline-viewport";

import { VisibleRecordingMoment } from "./recording-moments";

/**
 * The moments over the ruler, each a pin in its kind's colour that takes the
 * playhead to it. Read-only: a moment says where something happened, so it
 * is not dragged or cut, and goes with the part of the recording it is in.
 */
export function TimelineMomentPins({
  durationMs,
  moments,
  onSeek,
  viewport,
}: {
  /** The cut timeline's length, which the tooltips give times on. */
  durationMs: number;
  moments: readonly VisibleRecordingMoment[];
  onSeek: SeekHandler;
  viewport: TimelineViewportState;
}) {
  if (moments.length === 0) return null;
  return (
    <div className="pointer-events-none absolute inset-0 overflow-hidden">
      {moments.map(({ moment, output, source }) => {
        const left = (output - viewport.panOffset) * viewport.zoom;
        if (left < 0 || left > 1) return null;
        const label = `${moment.name}, ${formatDuration(output * durationMs)}`;
        return (
          // The tooltip anchors to the box its trigger wraps the pin in, so
          // the pin's place is set out here, around that box, where the
          // box can stretch to the pin's full height.
          <div
            className="absolute inset-y-0 flex -translate-x-1/2"
            key={`${moment.kindId}:${source.toString()}`}
            style={{ left: `${(left * 100).toString()}%` }}
          >
            <NativeTooltipTrigger tooltip={label}>
              <Button
                aria-label={`Go to ${moment.name} moment at ${formatDuration(output * durationMs)}`}
                className={cn(
                  "pointer-events-auto flex w-section cursor-default justify-center rounded-control outline-none",
                  focusStyles,
                  elementFocusVisible,
                )}
                onPress={() => {
                  onSeek(output, "start");
                  onSeek(output, "end");
                }}
              >
                <span
                  className="h-full w-tight rounded-full"
                  style={{ backgroundColor: moment.color }}
                />
              </Button>
            </NativeTooltipTrigger>
          </div>
        );
      })}
    </div>
  );
}
