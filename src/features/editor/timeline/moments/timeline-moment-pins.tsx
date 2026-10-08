// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "react-aria-components";

import { NativeTooltipTrigger } from "../../../../components/shared/native-tooltip/native-tooltip-trigger";
import { t } from "../../../../i18n/i18n";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";
import { formatDuration } from "../../duration";
import { RecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { SeekHandler } from "../timeline-seek";
import { TimelineViewportState } from "../timeline-viewport";

import { toggleMomentNote } from "./moment-note-popover";
import { RecordingMoment, VisibleRecordingMoment } from "./recording-moments";
import { useRulerCrowding } from "./use-ruler-crowding";

/** How much of a transcript a tooltip shows before it trails off. */
const TOOLTIP_TRANSCRIPT_CHARS = 60;

/** A pin's tooltip: its kind and time, and the start of what its note says
 * once that is known. */
const pinLabel = (moment: RecordingMoment, at: string) => {
  const transcript = moment.note?.transcript;
  // An empty transcript is a note with no speech, which the tooltip
  // describes as it does an untranscribed one.
  const text = transcript?.status === "ready" ? transcript.text.trim() : "";
  if (text) {
    const shown =
      text.length > TOOLTIP_TRANSCRIPT_CHARS
        ? `${text.slice(0, TOOLTIP_TRANSCRIPT_CHARS).trimEnd()}…`
        : text;
    return t("editor-timeline-pin-transcript", {
      moment: moment.name,
      text: shown,
      time: at,
    });
  }
  return moment.note
    ? t("editor-timeline-pin-note", { moment: moment.name, time: at })
    : t("editor-timeline-pin", { moment: moment.name, time: at });
};

/**
 * The moments over the ruler, each a pin in its kind's colour that takes the
 * playhead to it. Read-only: a moment says where something happened, so it
 * is not dragged or cut, and goes with the part of the recording it is in. A
 * pin with a voice note wears a head, and opens the note as it is pressed.
 * A pin that would run into a retime strip or a cut marker keeps to the
 * ruler's upper half, beside the ticks, so neither covers the other.
 */
export function TimelineMomentPins({
  durationMs,
  edit,
  moments,
  onSeek,
  viewport,
}: {
  /** The cut timeline's length, which the tooltips give times on. */
  durationMs: number;
  /** The cut timeline, for what the ruler draws under its ticks. */
  edit: RecordingTimelineEdit;
  moments: readonly VisibleRecordingMoment[];
  onSeek: SeekHandler;
  viewport: TimelineViewportState;
}) {
  const { isCrowded, probeRef, rootRef } = useRulerCrowding(
    edit,
    viewport,
    moments.length > 0,
  );
  if (moments.length === 0) return null;
  return (
    // Not clipped: a pin at either end of the timeline overhangs it by half
    // its width rather than losing half its head. Pins scrolled out of view
    // are left out below instead.
    <div className="pointer-events-none absolute inset-0" ref={rootRef}>
      {/* Half a pin and half a cut marker together: centres closer than
          this meet. */}
      <span
        aria-hidden
        className="invisible absolute w-[calc((var(--spacing-section)_+_var(--spacing-control-compact))/2)]"
        ref={probeRef}
      />
      {moments.map(({ moment, output }) => {
        const left = (output - viewport.panOffset) * viewport.zoom;
        if (left < 0 || left > 1) return null;
        const at = formatDuration(output * durationMs);
        const label = pinLabel(moment, at);
        return (
          // The tooltip anchors to the box its trigger wraps the pin in, so
          // the pin's place is set out here, around that box, where the
          // box can stretch to the pin's full height.
          <div
            className={`absolute flex -translate-x-1/2 ${isCrowded(output) ? "top-0 h-1/2" : "inset-y-0"}`}
            key={moment.index}
            style={{ left: `${(left * 100).toString()}%` }}
          >
            <NativeTooltipTrigger tooltip={label}>
              <Button
                aria-label={
                  moment.note
                    ? t("editor-timeline-go-to-note", {
                        moment: moment.name,
                        time: at,
                      })
                    : t("editor-timeline-go-to", {
                        moment: moment.name,
                        time: at,
                      })
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
                    edit.artifactId,
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
