// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useRef, useState } from "react";

import {
  RecordingTimelineEdit,
  recordingTimelineOutputToSource,
  recordingTimelineSourceEndToOutput,
  recordingTimelineSourceToOutput,
} from "../../timeline/editing/recording-timeline-edit";
import { TimelineRangeSelection } from "../../timeline/editing/timeline-blade";
import { Playhead } from "../../timeline/scrub-playhead";
import { SeekHandler } from "../../timeline/timeline-seek";

import { usePublishRecordingTranscriptPreview } from "./recording-transcript-channel";

const shareOf = (sourceMs: number, sourceDurationMs: number) =>
  sourceDurationMs > 0
    ? Math.max(0, Math.min(1, sourceMs / sourceDurationMs))
    : 0;

/** Where `sourceMs` lands on the timeline the edit plays, from 0 to 1. */
const outputAt = (
  edit: RecordingTimelineEdit | null | undefined,
  sourceMs: number,
  sourceDurationMs: number,
) => {
  const source = shareOf(sourceMs, sourceDurationMs);
  return edit ? recordingTimelineSourceToOutput(edit, source) : source;
};

/** Where a stretch ending at `sourceMs` stops on the timeline, from 0 to 1. */
const outputEndAt = (
  edit: RecordingTimelineEdit | null | undefined,
  sourceMs: number,
  sourceDurationMs: number,
) => {
  const source = shareOf(sourceMs, sourceDurationMs);
  return edit ? recordingTimelineSourceEndToOutput(edit, source) : source;
};

/**
 * Publishes the preview's seek and playhead for the Transcript panel: the
 * preview owns both, and the editor reaches them through the channel the way
 * the scene under the playhead is published. The playhead is published in
 * source milliseconds, mapped through the edit from the output position it
 * moves in.
 *
 * Returns the words chosen in the panel as a stretch of the timeline, for
 * the timeline to show; null while none are, or while every chosen word is
 * cut and so takes no time.
 */
export function useRecordingTranscriptPreview({
  edit,
  playhead,
  seek,
  sourceDurationMs,
}: {
  edit: RecordingTimelineEdit | null | undefined;
  playhead: Playhead;
  seek: SeekHandler;
  sourceDurationMs: number;
}): TimelineRangeSelection | null {
  // Read when the playhead moves rather than when the panel subscribed, so a
  // cut made since still maps the playhead to the right word.
  const editRef = useRef(edit);
  editRef.current = edit;
  const [highlight, setHighlight] = useState<{
    endMs: number;
    startMs: number;
  } | null>(null);
  usePublishRecordingTranscriptPreview("recording", {
    highlight: setHighlight,
    seek: (sourceMs) => {
      const output = outputAt(edit, sourceMs, sourceDurationMs);
      seek(output, "start");
      seek(output, "end");
    },
    subscribe: (listener) =>
      playhead.subscribe((_seconds, ratio) => {
        const latest = editRef.current;
        const source = latest
          ? recordingTimelineOutputToSource(latest, ratio)
          : ratio;
        listener(Math.round(source * sourceDurationMs));
      }),
  });
  return useMemo(() => {
    if (!highlight) return null;
    const start = outputAt(edit, highlight.startMs, sourceDurationMs);
    const end = outputEndAt(edit, highlight.endMs, sourceDurationMs);
    return end > start ? { end, start } : null;
  }, [edit, highlight, sourceDurationMs]);
}
