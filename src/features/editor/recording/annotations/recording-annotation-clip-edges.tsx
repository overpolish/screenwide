// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PointerEvent } from "react";

import {
  RecordingTimelineEdit,
  recordingTimelineRetainedDuration,
  recordingTimelineSourceToOutput,
} from "../../timeline/editing/recording-timeline-edit";
import { SeekHandler } from "../../timeline/timeline-seek";

import { previewedWhole } from "./recording-annotation-drag-draft";
import { trimRecordingAnnotationClips } from "./recording-annotation-geometry";
import { RecordingAnnotationClip } from "./recording-annotations";

type Edge = "startMs" | "endMs";

/**
 * The trim handles at a clip's two ends. A press is the lane's to turn into a
 * drag, and shows the annotation whole at the edge it took hold of: the frame
 * under a trim handle is the one frame the annotation is barely there, which
 * is no use for deciding where the handle belongs. The arrow keys step the
 * edge a tenth of a second, a whole second with Shift, trimming every clip in
 * `ids` with it. A fragment that continues into its neighbour across a cut
 * has no handle on that side: the clip does not end there.
 */
export function RecordingAnnotationClipEdges({
  clip,
  clips,
  continuedByNext,
  continuesPrevious,
  edit,
  ids,
  label,
  onChange,
  onPress,
  onSeek,
  sourceDurationMs,
}: {
  clip: RecordingAnnotationClip;
  clips: RecordingAnnotationClip[];
  continuedByNext: boolean;
  continuesPrevious: boolean;
  edit: RecordingTimelineEdit;
  /** The clips a trim of this one trims together: it, or the choice it is
   * one of. */
  ids: ReadonlySet<string>;
  label: string;
  onChange: (clips: RecordingAnnotationClip[]) => void;
  onPress: (edge: Edge, event: PointerEvent) => void;
  sourceDurationMs: number;
  onSeek?: SeekHandler;
}) {
  return (["startMs", "endMs"] as const).map((edge) => {
    if (
      (edge === "startMs" && continuesPrevious) ||
      (edge === "endMs" && continuedByNext)
    )
      return null;
    return (
      <button
        aria-label={`${edge === "startMs" ? "Start" : "End"} of ${label}`}
        className={`absolute inset-y-0 w-control-inset cursor-ew-resize focus-visible:bg-primary focus-visible:outline-none ${edge === "startMs" ? "left-0" : "right-0"}`}
        key={edge}
        onClick={(event) => {
          event.stopPropagation();
        }}
        onKeyDown={(event) => {
          if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
          event.preventDefault();
          event.stopPropagation();
          const output = recordingTimelineSourceToOutput(
            edit,
            clip[edge] / sourceDurationMs,
          );
          const step =
            (event.shiftKey ? 1000 : 100) /
            (sourceDurationMs * recordingTimelineRetainedDuration(edit));
          onChange(
            trimRecordingAnnotationClips({
              clips,
              edge,
              edit,
              id: clip.annotation.id,
              ids,
              output: output + (event.key === "ArrowLeft" ? -step : step),
              sourceDurationMs,
            }),
          );
        }}
        onPointerDown={(event) => {
          if (event.button !== 0) return;
          event.stopPropagation();
          onPress(edge, event);
          onSeek?.(
            recordingTimelineSourceToOutput(
              edit,
              (edge === "endMs" ? clip.endMs - 1 : clip.startMs) /
                sourceDurationMs,
            ),
            "start",
            previewedWhole(clips, clip.annotation.id),
          );
        }}
        type="button"
      />
    );
  });
}
