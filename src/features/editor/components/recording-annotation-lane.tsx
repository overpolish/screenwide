// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PencilLine } from "lucide-react";

import { annotationLaneLabel } from "../annotations";
import { resizeRecordingAnnotationClip } from "../recording-annotation-geometry";
import { RecordingAnnotationClip } from "../recording-annotations";
import {
  RecordingTimelineEdit,
  recordingTimelineSourceToOutput,
  recordingTimelineRetainedDuration,
} from "../recording-timeline-edit";

import { recordingAnnotationRows } from "./recording-annotation-layout";
import {
  TIMED_LANE_ROW_HEIGHT_PX,
  timedLaneFragmentBox,
} from "./timed-lane-layout";
import { SeekHandler } from "./timeline-seek";
import { TimelineTrackHeader } from "./timeline-track-header";
import { TimelineViewportState } from "./timeline-viewport";
import { TimelineViewportContent } from "./timeline-viewport-content";
import {
  previewedWhole,
  useRecordingAnnotationDrag,
} from "./use-recording-annotation-drag";

export function RecordingAnnotationLane({
  clips,
  edit,
  onChange,
  onPreview,
  onSeek,
  onSelect,
  selectedId,
  sourceDurationMs,
  viewport,
}: {
  clips: RecordingAnnotationClip[];
  edit: RecordingTimelineEdit;
  onChange: (clips: RecordingAnnotationClip[]) => void;
  onSelect: (id: string) => void;
  selectedId: string | null;
  sourceDurationMs: number;
  viewport: TimelineViewportState;
  onPreview?: (clips: RecordingAnnotationClip[] | null) => void;
  onSeek?: SeekHandler;
}) {
  const { beginDrag, draft, laneRef, movedRef } = useRecordingAnnotationDrag({
    clips,
    edit,
    onCommit: onChange,
    onPreview,
    onSeek,
    onSelect,
    sourceDurationMs,
    viewport,
  });
  const laidOut = recordingAnnotationRows(
    draft ?? clips,
    edit,
    sourceDurationMs,
  );
  return (
    <div className="flex items-center gap-section">
      <TimelineTrackHeader
        icon={<PencilLine />}
        isSelected={selectedId !== null}
        label="Annotations"
      />
      <div
        className="relative min-w-0 grow overflow-hidden rounded-control bg-fill-tertiary"
        ref={laneRef}
        style={{ height: laidOut.rowCount * TIMED_LANE_ROW_HEIGHT_PX }}
      >
        <TimelineViewportContent viewport={viewport}>
          {laidOut.fragments.map((fragment) => {
            const clip = fragment.item;
            // A counter is called by the number it shows; an arrow has no
            // name of its own, so it is called by its place in the lane.
            const label = annotationLaneLabel(
              clip.annotation,
              clips.findIndex(
                (item) => item.annotation.id === clip.annotation.id,
              ),
            );
            const selected = clip.annotation.id === selectedId;
            return (
              <div
                className={`absolute overflow-hidden rounded-control text-footnote ${fragment.continuesPrevious ? "rounded-l-none" : ""} ${fragment.continuedByNext ? "rounded-r-none" : ""} ${selected ? "bg-primary-surface text-primary-fg" : "bg-fill-secondary text-content-fg"}`}
                key={fragment.fragmentId}
                style={{
                  ...timedLaneFragmentBox(fragment.row),
                  left: `${String(fragment.outputStart * 100)}%`,
                  minWidth: 6,
                  width: `${String((fragment.outputEnd - fragment.outputStart) * 100)}%`,
                }}
              >
                <button
                  aria-label={label}
                  aria-pressed={selected}
                  className="h-full w-full truncate px-control-inset text-left focus-visible:outline-2 focus-visible:outline-primary"
                  onClick={(event) => {
                    event.stopPropagation();
                    if (movedRef.current) {
                      event.preventDefault();
                      movedRef.current = false;
                    } else onSelect(clip.annotation.id);
                  }}
                  onPointerDown={(event) => {
                    if (event.button !== 0) return;
                    event.stopPropagation();
                    beginDrag({
                      clientX: event.clientX,
                      edge: "body",
                      id: clip.annotation.id,
                    });
                  }}
                  type="button"
                >
                  {fragment.showLabel ? label : null}
                </button>
                {(["startMs", "endMs"] as const).map((edge) => {
                  if (
                    (edge === "startMs" && fragment.continuesPrevious) ||
                    (edge === "endMs" && fragment.continuedByNext)
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
                        if (
                          event.key !== "ArrowLeft" &&
                          event.key !== "ArrowRight"
                        )
                          return;
                        event.preventDefault();
                        event.stopPropagation();
                        const output = recordingTimelineSourceToOutput(
                          edit,
                          clip[edge] / sourceDurationMs,
                        );
                        const step =
                          (event.shiftKey ? 1000 : 100) /
                          (sourceDurationMs *
                            recordingTimelineRetainedDuration(edit));
                        onChange(
                          resizeRecordingAnnotationClip({
                            clips,
                            edge,
                            edit,
                            id: clip.annotation.id,
                            output:
                              output +
                              (event.key === "ArrowLeft" ? -step : step),
                            sourceDurationMs,
                          }),
                        );
                      }}
                      onPointerDown={(event) => {
                        if (event.button !== 0) return;
                        event.stopPropagation();
                        onSelect(clip.annotation.id);
                        beginDrag({
                          clientX: event.clientX,
                          edge,
                          id: clip.annotation.id,
                        });
                        // The press alone shows the annotation whole at the
                        // edge it took hold of: the frame under a trim handle
                        // is the one frame the annotation is barely there,
                        // which is no use for deciding where the handle
                        // belongs.
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
                })}
              </div>
            );
          })}
        </TimelineViewportContent>
      </div>
    </div>
  );
}
