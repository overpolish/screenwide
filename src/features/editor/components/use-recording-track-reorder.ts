// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
import { PointerEvent as ReactPointerEvent, useRef, useState } from "react";

import { RecordingVideoTrackId } from "../types";

import { TIMELINE_TRACK_SWITCH_SELECTOR } from "./timeline-track-header";
import { useRecordingTrackMenu } from "./use-recording-track-menu";
const DRAG_THRESHOLD_PX = 4;
export function useRecordingTrackReorder(
  videoTrackOrder: RecordingVideoTrackId[],
  videoRows: { trackId: RecordingVideoTrackId }[],
  onVideoTrackOrderChange?: (tracks: RecordingVideoTrackId[]) => void,
) {
  const [drag, setDrag] = useState<{
    dropIndex: number;
    source: RecordingVideoTrackId;
  } | null>(null);
  const dragRef = useRef<{
    dropIndex: number;
    pointerId: number;
    source: RecordingVideoTrackId;
    startY: number;
    started: boolean;
  } | null>(null);
  const rowElementsRef = useRef(
    new Map<RecordingVideoTrackId, HTMLDivElement>(),
  );
  const applyOrder = (source: RecordingVideoTrackId, dropIndex: number) => {
    const sourceIndex = videoTrackOrder.indexOf(source);
    if (sourceIndex === -1) return;
    const next = videoTrackOrder.filter((track) => track !== source);
    const insertionIndex = Math.max(
      0,
      Math.min(
        next.length,
        dropIndex > sourceIndex ? dropIndex - 1 : dropIndex,
      ),
    );
    next.splice(insertionIndex, 0, source);
    if (next.some((track, index) => track !== videoTrackOrder[index]))
      onVideoTrackOrderChange?.(next);
  };
  const moveTrack = (
    track: RecordingVideoTrackId,
    direction: "backward" | "forward",
  ) => {
    const index = videoTrackOrder.indexOf(track);
    const nextIndex = direction === "forward" ? index - 1 : index + 1;
    if (index === -1 || nextIndex < 0 || nextIndex >= videoTrackOrder.length)
      return;
    const next = [...videoTrackOrder];
    [next[index], next[nextIndex]] = [next[nextIndex], next[index]];
    onVideoTrackOrderChange?.(next);
  };
  const openTrackMenu = useRecordingTrackMenu(moveTrack);
  const beginDrag =
    (source: RecordingVideoTrackId) =>
    (event: ReactPointerEvent<HTMLDivElement>) => {
      if (event.button !== 0 || videoRows.length < 2) return;
      // The switch acts on the track it sits in, so a press on it is never
      // the start of a reorder: capturing the pointer here would take the
      // switch's click with it. The rest of the header is the drag handle,
      // the button that selects the track included.
      if (
        event.target instanceof Element &&
        event.target.closest(TIMELINE_TRACK_SWITCH_SELECTOR)
      )
        return;
      // Only the press point is recorded here. Capturing the pointer now
      // would redirect the pointerup and the click that follows it to this
      // row, and the header's button would never complete its press - the
      // same mechanism that once swallowed the switch. The capture is taken
      // when the press turns into a reorder, and a press that never moves
      // stays an ordinary press on the button that selects the track.
      dragRef.current = {
        dropIndex: videoTrackOrder.indexOf(source),
        pointerId: event.pointerId,
        source,
        startY: event.clientY,
        started: false,
      };
    };
  const updateDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    const active = dragRef.current;
    if (!active) return;
    if (
      !active.started &&
      Math.abs(event.clientY - active.startY) <= DRAG_THRESHOLD_PX
    )
      return;
    if (!active.started) {
      // From here the row owns the pointer: the reorder has started, so the
      // press it grew out of is meant to be cancelled.
      event.currentTarget.setPointerCapture(active.pointerId);
    }
    active.started = true;
    event.preventDefault();
    let dropIndex = videoRows.length;
    for (let index = 0; index < videoRows.length; index += 1) {
      const row = rowElementsRef.current.get(videoRows[index].trackId);
      if (!row) continue;
      const bounds = row.getBoundingClientRect();
      if (event.clientY < bounds.top + bounds.height / 2) {
        dropIndex = index;
        break;
      }
    }
    active.dropIndex = dropIndex;
    setDrag({ dropIndex, source: active.source });
  };
  const finishDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    const active = dragRef.current;
    dragRef.current = null;
    if (active?.started) applyOrder(active.source, active.dropIndex);
    setDrag(null);
    if (event.currentTarget.hasPointerCapture(event.pointerId))
      event.currentTarget.releasePointerCapture(event.pointerId);
  };
  const cancelDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    dragRef.current = null;
    setDrag(null);
    if (event.currentTarget.hasPointerCapture(event.pointerId))
      event.currentTarget.releasePointerCapture(event.pointerId);
  };
  return {
    beginDrag,
    cancelDrag,
    drag,
    finishDrag,
    openTrackMenu,
    rowElementsRef,
    updateDrag,
  };
}
