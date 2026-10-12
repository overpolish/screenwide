// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  PointerEvent as ReactPointerEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";

import { ownsTextEditingKeys } from "../../shortcuts/keyboard-target";

import {
  layoutRecordingTimelineSegments,
  RecordingTimelineEdit,
} from "./recording-timeline-edit";
import { neighbourBoundary } from "./recording-timeline-move";

/** How far a press travels before it moves the segment rather than picks
 * it, as the timeline's other drags do. */
const DRAG_SLOP_PX = 4;

/**
 * The boundary nearest `outputPosition` a segment can be dropped on, with
 * where it is in output time; null where the nearest one would leave the
 * segment where it is.
 */
export function nearestSegmentBoundary(
  edit: RecordingTimelineEdit,
  segmentId: number,
  outputPosition: number,
): { boundary: number; outputPosition: number } | null {
  const layout = layoutRecordingTimelineSegments(edit);
  const from = layout.findIndex((segment) => segment.id === segmentId);
  if (from < 0) return null;
  const positions = [...layout.map((segment) => segment.outputStart), 1];
  let boundary = 0;
  positions.forEach((position, index) => {
    if (
      Math.abs(position - outputPosition) <
      Math.abs(positions[boundary] - outputPosition)
    )
      boundary = index;
  });
  return boundary === from || boundary === from + 1
    ? null
    : { boundary, outputPosition: positions[boundary] };
}

/** Keeps the click a drag ends with from reaching the segment or the lane
 * under the pointer: it was a move, not a pick or a deselect. */
const swallowClick = () => {
  const swallow = (event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();
  };
  window.addEventListener("click", swallow, { capture: true, once: true });
  window.setTimeout(() => {
    window.removeEventListener("click", swallow, true);
  }, 0);
};

/**
 * Dragging a segment's body to another place on the timeline, and Option
 * Command (Ctrl Alt on Windows) with Left or Right to swap the chosen
 * segment with its neighbour. Every linked track moves together. While a
 * drag is under way the boundary it would drop on is shown through every
 * lane with the snap guide; Escape abandons it.
 */
export function useTimelineSegmentMove({
  edit,
  keysEnabled,
  onMove,
  selectedSegmentId,
  setGuidePosition,
}: {
  edit: RecordingTimelineEdit;
  keysEnabled: boolean;
  onMove: (segmentId: number, boundary: number) => void;
  selectedSegmentId: number | null;
  setGuidePosition: (outputPosition: number | null) => void;
}) {
  const [movingSegmentId, setMovingSegmentId] = useState<number | null>(null);
  const latestRef = useRef({
    edit,
    onMove,
    selectedSegmentId,
    setGuidePosition,
  });
  latestRef.current = { edit, onMove, selectedSegmentId, setGuidePosition };
  const detachRef = useRef<() => void>(() => undefined);
  useEffect(
    () => () => {
      detachRef.current();
    },
    [],
  );

  const begin = useCallback(
    (
      segmentId: number,
      event: ReactPointerEvent<HTMLElement>,
      outputPositionAt: (clientX: number) => number,
    ) => {
      if (event.button !== 0) return;
      detachRef.current();
      const startX = event.clientX;
      let moved = false;
      let boundary: number | null = null;
      const show = (drop: ReturnType<typeof nearestSegmentBoundary>) => {
        boundary = drop?.boundary ?? null;
        latestRef.current.setGuidePosition(drop?.outputPosition ?? null);
      };
      const move = (moveEvent: PointerEvent) => {
        if (!moved && Math.abs(moveEvent.clientX - startX) < DRAG_SLOP_PX)
          return;
        if (!moved) setMovingSegmentId(segmentId);
        moved = true;
        show(
          nearestSegmentBoundary(
            latestRef.current.edit,
            segmentId,
            outputPositionAt(moveEvent.clientX),
          ),
        );
      };
      const finish = () => {
        detachRef.current();
        if (moved) show(null);
        setMovingSegmentId(null);
      };
      const release = () => {
        const dropped = moved ? boundary : null;
        if (moved) swallowClick();
        finish();
        if (dropped !== null) latestRef.current.onMove(segmentId, dropped);
      };
      const cancel = (keyEvent: KeyboardEvent) => {
        if (keyEvent.key !== "Escape" || !moved) return;
        keyEvent.preventDefault();
        keyEvent.stopPropagation();
        finish();
      };
      detachRef.current = () => {
        detachRef.current = () => undefined;
        window.removeEventListener("pointermove", move, true);
        window.removeEventListener("pointerup", release, true);
        window.removeEventListener("pointercancel", finish, true);
        window.removeEventListener("keydown", cancel, true);
      };
      window.addEventListener("pointermove", move, true);
      window.addEventListener("pointerup", release, true);
      window.addEventListener("pointercancel", finish, true);
      window.addEventListener("keydown", cancel, true);
    },
    [],
  );

  useEffect(() => {
    if (!keysEnabled) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (
        !event.altKey ||
        !(event.metaKey || event.ctrlKey) ||
        event.shiftKey ||
        event.isComposing ||
        (event.code !== "ArrowLeft" && event.code !== "ArrowRight") ||
        ownsTextEditingKeys(event.target)
      )
        return;
      const { edit, onMove, selectedSegmentId } = latestRef.current;
      if (selectedSegmentId === null) return;
      event.preventDefault();
      event.stopPropagation();
      const boundary = neighbourBoundary(
        edit.segments.length,
        edit.segments.findIndex((segment) => segment.id === selectedSegmentId),
        event.code === "ArrowRight" ? 1 : -1,
      );
      if (boundary !== null) onMove(selectedSegmentId, boundary);
    };
    window.addEventListener("keydown", onKeyDown, true);
    return () => {
      window.removeEventListener("keydown", onKeyDown, true);
    };
  }, [keysEnabled]);

  return { begin, movingSegmentId };
}
