// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PencilLine } from "lucide-react";

import { resizeRecordingAnnotationClip } from "../recording-annotation-geometry";
import { RecordingAnnotationClip } from "../recording-annotations";
import {
  RecordingTimelineEdit,
  recordingTimelineSourceToOutput,
  recordingTimelineRetainedDuration,
} from "../recording-timeline-edit";

import { recordingAnnotationRows } from "./recording-annotation-layout";
import { SeekHandler } from "./scrub-timeline";
import {
  TIMED_LANE_ROW_HEIGHT_PX,
  timedLaneFragmentBox,
} from "./timed-lane-layout";
import { TimelineTrackHeader } from "./timeline-track-header";
import { TimelineViewportState } from "./timeline-viewport";
import { TimelineViewportContent } from "./timeline-viewport-content";
import { useRecordingAnnotationDrag } from "./use-recording-annotation-drag";


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
  const {
    cancel,
    draft,
    draftRef,
    dragRef,
    finish,
    laneRef,
    movedRef,
    setDraft,
    update,
  } = useRecordingAnnotationDrag({
    edit,
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
            const label = `Arrow ${String(clips.findIndex((item) => item.annotation.id === clip.annotation.id) + 1)}`;
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
                  onLostPointerCapture={cancel}
                  onPointerCancel={cancel}
                  onPointerDown={(event) => {
                    if (event.button !== 0) return;
                    event.stopPropagation();
                    movedRef.current = false;
                    dragRef.current = {
                      edge: "body",
                      id: clip.annotation.id,
                      moved: false,
                      original: clips,
                      startX: event.clientX,
                    };
                    draftRef.current = clips;
                    setDraft(clips);
                    event.currentTarget.setPointerCapture(event.pointerId);
                  }}
                  onPointerMove={(event) => {
                    update(event.clientX);
                  }}
                  onPointerUp={(event) => {
                    if (!dragRef.current) return;
                    update(event.clientX);
                    const next = draftRef.current;
                    const moved = movedRef.current;
                    finish();
                    if (event.currentTarget.hasPointerCapture(event.pointerId))
                      event.currentTarget.releasePointerCapture(
                        event.pointerId,
                      );
                    if (next && moved) onChange(next);
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
                      onLostPointerCapture={cancel}
                      onPointerCancel={cancel}
                      onPointerDown={(event) => {
                        if (event.button !== 0) return;
                        event.stopPropagation();
                        onSelect(clip.annotation.id);
                        dragRef.current = {
                          edge,
                          id: clip.annotation.id,
                          moved: false,
                          original: clips,
                          startX: event.clientX,
                        };
                        draftRef.current = clips;
                        setDraft(clips);
                        onSeek?.(
                          recordingTimelineSourceToOutput(
                            edit,
                            (edge === "endMs" ? clip.endMs - 1 : clip.startMs) /
                              sourceDurationMs,
                          ),
                          "start",
                          clips,
                        );
                        event.currentTarget.setPointerCapture(event.pointerId);
                      }}
                      onPointerMove={(event) => {
                        update(event.clientX);
                      }}
                      onPointerUp={(event) => {
                        if (!dragRef.current) return;
                        update(event.clientX);
                        const next = draftRef.current;
                        finish();
                        if (
                          event.currentTarget.hasPointerCapture(event.pointerId)
                        )
                          event.currentTarget.releasePointerCapture(
                            event.pointerId,
                          );
                        if (next) onChange(next);
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
