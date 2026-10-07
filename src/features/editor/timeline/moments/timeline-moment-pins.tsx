// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "react-aria-components";

import { NativeTooltipTrigger } from "../../../../components/shared/native-tooltip/native-tooltip-trigger";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";
import { formatDuration } from "../../duration";
import { SeekHandler } from "../timeline-seek";
import { TimelineViewportState } from "../timeline-viewport";

import { toggleMomentNote } from "./moment-note-popover";
import { VisibleRecordingMoment } from "./recording-moments";

/**
 * The moments over the ruler, each a pin in its kind's colour that takes the
 * playhead to it. Read-only: a moment says where something happened, so it
 * is not dragged or cut, and goes with the part of the recording it is in. A
 * pin with a voice note wears a head, and opens the note as it is pressed.
 */
export function TimelineMomentPins({
  artifactId,
  durationMs,
  moments,
  onSeek,
  viewport,
}: {
  artifactId: number;
  /** The cut timeline's length, which the tooltips give times on. */
  durationMs: number;
  moments: readonly VisibleRecordingMoment[];
  onSeek: SeekHandler;
  viewport: TimelineViewportState;
}) {
  if (moments.length === 0) return null;
  return (
    // Not clipped: a pin at either end of the timeline overhangs it by half
    // its width rather than losing half its head. Pins scrolled out of view
    // are left out below instead.
    <div className="pointer-events-none absolute inset-0">
      {moments.map(({ moment, output }) => {
        const left = (output - viewport.panOffset) * viewport.zoom;
        if (left < 0 || left > 1) return null;
        const at = formatDuration(output * durationMs);
        const label = moment.note
          ? `${moment.name}, ${at}, with a voice note`
          : `${moment.name}, ${at}`;
        return (
          // The tooltip anchors to the box its trigger wraps the pin in, so
          // the pin's place is set out here, around that box, where the
          // box can stretch to the pin's full height.
          <div
            className="absolute inset-y-0 flex -translate-x-1/2"
            key={moment.index}
            style={{ left: `${(left * 100).toString()}%` }}
          >
            <NativeTooltipTrigger tooltip={label}>
              <Button
                aria-label={
                  moment.note
                    ? `Go to ${moment.name} moment at ${at} and open its voice note`
                    : `Go to ${moment.name} moment at ${at}`
                }
                className={cn(
                  "pointer-events-auto relative flex w-section cursor-default justify-center rounded-control outline-none",
                  focusStyles,
                  elementFocusVisible,
                )}
                onPress={(event) => {
                  onSeek(output, "start");
                  onSeek(output, "end");
                  void toggleMomentNote(
                    artifactId,
                    moment,
                    event.target.getBoundingClientRect(),
                  );
                }}
              >
                <span
                  className="h-full w-tight rounded-full"
                  style={{ backgroundColor: moment.color }}
                />
                {moment.note ? (
                  <span
                    className="absolute top-0 left-1/2 size-control-inset -translate-x-1/2 rounded-full"
                    style={{ backgroundColor: moment.color }}
                  />
                ) : null}
              </Button>
            </NativeTooltipTrigger>
          </div>
        );
      })}
    </div>
  );
}
