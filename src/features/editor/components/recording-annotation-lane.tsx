// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PencilLine } from "lucide-react";
import { MouseEvent, PointerEvent } from "react";

import { boundsAnchor, pointerAnchor } from "../../popup-panel/use-popup-menu";
import { annotationLaneLabel } from "../annotation-kinds";
import { resizeRecordingAnnotationClip } from "../recording-annotation-geometry";
import { isPinnable, withoutPinKeyframe } from "../recording-annotation-pins";
import { RecordingAnnotationClip } from "../recording-annotations";
import {
  RecordingTimelineEdit,
  recordingTimelineSourceToOutput,
  recordingTimelineRetainedDuration,
} from "../recording-timeline-edit";
import { RecordingPinStatus } from "../use-recording-pin-status";

import { recordingAnnotationRows } from "./recording-annotation-layout";
import {
  RecordingAnnotationPinBadge,
  RecordingAnnotationPinKeyframes,
  RecordingAnnotationPinStretches,
} from "./recording-annotation-pin-overlay";
import {
  TIMED_LANE_ROW_HEIGHT_PX,
  timedLaneFragmentBox,
} from "./timed-lane-layout";
import { SeekHandler } from "./timeline-seek";
import { TimelineTrackHeader } from "./timeline-track-header";
import { TimelineViewportState } from "./timeline-viewport";
import { TimelineViewportContent } from "./timeline-viewport-content";
import { useAnnotationClipMenu } from "./use-annotation-clip-menu";
import { usePinKeyframeMenu } from "./use-pin-keyframe-menu";
import {
  previewedWhole,
  useRecordingAnnotationDrag,
} from "./use-recording-annotation-drag";

export function RecordingAnnotationLane({
  clips,
  edit,
  onChange,
  onClearPinCorrections,
  onPinnedChange,
  onPreview,
  onSeek,
  onSelect,
  pinStatus,
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
  /** Take away every place the annotation was put by hand. */
  onClearPinCorrections?: (id: string) => void;
  /** Pin the annotation to the content under it, or let it go. Without it,
   * the lane offers no pinning. */
  onPinnedChange?: (id: string, pinned: boolean) => void;
  onPreview?: (clips: RecordingAnnotationClip[] | null) => void;
  onSeek?: SeekHandler;
  /** How each pinned clip's path is coming along, by annotation id. */
  pinStatus?: ReadonlyMap<string, RecordingPinStatus>;
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
  const deleteKeyframe = (id: string, ms: number) => {
    onChange(
      clips.map((clip) =>
        clip.annotation.id === id && clip.pin
          ? { ...clip, pin: withoutPinKeyframe(clip.pin, ms) }
          : clip,
      ),
    );
  };
  const openKeyframeMenu = usePinKeyframeMenu(deleteKeyframe);
  const openClipMenu = useAnnotationClipMenu({
    onClearCorrections: (id) => onClearPinCorrections?.(id),
    onPinnedChange: (id, pinned) => onPinnedChange?.(id, pinned),
  });
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
            const status = pinStatus?.get(clip.annotation.id);
            const block = { edit, fragment, sourceDurationMs };
            const pinnable = onPinnedChange !== undefined && isPinnable(clip);
            const clickBody = (event: MouseEvent) => {
              event.stopPropagation();
              if (movedRef.current) {
                event.preventDefault();
                movedRef.current = false;
              } else onSelect(clip.annotation.id);
            };
            const pressBody = (event: PointerEvent) => {
              if (event.button !== 0) return;
              event.stopPropagation();
              beginDrag({
                clientX: event.clientX,
                edge: "body",
                id: clip.annotation.id,
              });
            };
            return (
              <div
                className={`absolute overflow-hidden rounded-control text-footnote ${fragment.continuesPrevious ? "rounded-l-none" : ""} ${fragment.continuedByNext ? "rounded-r-none" : ""} ${selected ? "bg-primary-surface text-primary-fg" : "bg-fill-secondary text-content-fg"}`}
                key={fragment.fragmentId}
                onContextMenu={(event) => {
                  if (!pinnable) return;
                  event.preventDefault();
                  void openClipMenu(
                    pointerAnchor(event.clientX, event.clientY),
                    clip,
                  );
                }}
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
                  onClick={clickBody}
                  // The menu key opens the right-click menu from the
                  // keyboard, hung off the block it acts on.
                  onKeyDown={(event) => {
                    const menuKey =
                      event.key === "ContextMenu" ||
                      (event.key === "F10" && event.shiftKey);
                    if (!menuKey || !pinnable) return;
                    event.preventDefault();
                    event.stopPropagation();
                    void openClipMenu(
                      boundsAnchor(event.currentTarget.getBoundingClientRect()),
                      clip,
                    );
                  }}
                  onPointerDown={pressBody}
                  type="button"
                >
                  {fragment.showLabel ? label : null}
                </button>
                {selected && clip.pin ? (
                  <>
                    <RecordingAnnotationPinStretches
                      {...block}
                      onBodyClick={clickBody}
                      onBodyPointerDown={pressBody}
                      status={status}
                    />
                    <RecordingAnnotationPinKeyframes
                      {...block}
                      label={label}
                      onDeleteKeyframe={(ms) => {
                        deleteKeyframe(clip.annotation.id, ms);
                      }}
                      onKeyframeMenu={(point, ms, isPinnedFrame) => {
                        void openKeyframeMenu({
                          annotationId: clip.annotation.id,
                          isPinnedFrame,
                          ms,
                          point,
                        });
                      }}
                      onSeek={onSeek}
                      pin={clip.pin}
                    />
                  </>
                ) : null}
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
                {clip.pin ? (
                  <RecordingAnnotationPinBadge
                    label={label}
                    selected={selected}
                    status={status}
                  />
                ) : null}
              </div>
            );
          })}
        </TimelineViewportContent>
      </div>
    </div>
  );
}
