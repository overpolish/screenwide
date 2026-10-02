// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import {
  RecordingTimelineEdit,
  recordingTimelineSourceToOutput,
} from "../../timeline/editing/recording-timeline-edit";
import { SeekHandler } from "../../timeline/timeline-seek";
import {
  beginTimelineSnapGesture,
  nearestTimelineSnapTarget,
  TIMELINE_SNAP_THRESHOLD_PX,
  TimelineSnapGesture,
  timelineSnapRangeShift,
  useTimelineSnap,
} from "../../timeline/timeline-snap";
import {
  TimelineViewportState,
  timelineXToFraction,
} from "../../timeline/timeline-viewport";

import {
  moveRecordingSceneClip,
  RecordingSceneClip,
  resizeRecordingSceneClip,
} from "./recording-scenes";

export type RecordingSceneDragEdge = "body" | "endMs" | "startMs";

/** How far a press travels before it is a drag rather than a click. */
const DRAG_SLOP_PX = 4;

type Drag = {
  edge: RecordingSceneDragEdge;
  id: string;
  moved: boolean;
  original: RecordingSceneClip[];
  snap: TimelineSnapGesture;
  startX: number;
};

/**
 * A press on a scene clip, owned until the pointer is released: the body
 * slides the clip in time and an edge trims it, both stopping at the clips on
 * either side. A trim parks the preview at the edge being moved. The draft
 * is shown in the preview as it changes and committed on release. The moves
 * and the release are listened for on the window, so a pointer that leaves
 * the lane mid-drag keeps dragging, and Escape abandons the drag.
 */
export function useRecordingSceneDrag({
  clips,
  edit,
  onCommit,
  onDraftChange,
  onSeek,
  sourceDurationMs,
  viewport,
}: {
  clips: RecordingSceneClip[];
  edit: RecordingTimelineEdit;
  onCommit: (clips: RecordingSceneClip[]) => void;
  sourceDurationMs: number;
  viewport: TimelineViewportState;
  /** Shows the draft in the preview, or the committed clips for null. */
  onDraftChange?: (clips: RecordingSceneClip[] | null) => void;
  onSeek?: SeekHandler;
}) {
  const [draft, setDraft] = useState<RecordingSceneClip[] | null>(null);
  const draftRef = useRef<RecordingSceneClip[] | null>(null);
  const dragRef = useRef<Drag | null>(null);
  // Read by the click that follows a release: a press that slid a clip was a
  // drag, and must not also count as a click on it.
  const movedRef = useRef(false);
  const laneRef = useRef<HTMLDivElement>(null);
  const snap = useTimelineSnap();
  const detachRef = useRef<() => void>(() => undefined);
  const seekEdge = (
    next: RecordingSceneClip[],
    drag: Drag,
    phase: "end" | "move",
  ) => {
    const clip = next.find((item) => item.id === drag.id);
    if (!clip || drag.edge === "body" || sourceDurationMs <= 0) return;
    const edgeMs = drag.edge === "endMs" ? clip.endMs - 1 : clip.startMs;
    onSeek?.(
      recordingTimelineSourceToOutput(edit, edgeMs / sourceDurationMs),
      phase,
    );
  };
  const reset = () => {
    dragRef.current?.snap.showGuide(null);
    dragRef.current = null;
    draftRef.current = null;
    setDraft(null);
  };
  const update = (clientX: number) => {
    const drag = dragRef.current;
    const bounds = laneRef.current?.getBoundingClientRect();
    if (!drag || !bounds || sourceDurationMs <= 0) return;
    if (!drag.moved && Math.abs(clientX - drag.startX) < DRAG_SLOP_PX) return;
    drag.moved = true;
    movedRef.current = true;
    const outputWidthPx = viewport.zoom * bounds.width;
    // Retaken every move: a wheel zoom mid-drag changes what 8px spans.
    drag.snap.threshold = TIMELINE_SNAP_THRESHOLD_PX / outputWidthPx;
    let next: RecordingSceneClip[];
    if (drag.edge === "body") {
      const clip = drag.original.find((item) => item.id === drag.id);
      if (!clip) return;
      const shift = (clientX - drag.startX) / outputWidthPx;
      const start = recordingTimelineSourceToOutput(
        edit,
        clip.startMs / sourceDurationMs,
      );
      const end = recordingTimelineSourceToOutput(
        edit,
        clip.endMs / sourceDurationMs,
      );
      const snapped = timelineSnapRangeShift(
        drag.snap,
        start + shift,
        end + shift,
      );
      drag.snap.showGuide(snapped?.target ?? null);
      next = moveRecordingSceneClip({
        clips: drag.original,
        deltaOutput: shift + (snapped?.shift ?? 0),
        edit,
        id: drag.id,
        sourceDurationMs,
      });
    } else {
      const reached = timelineXToFraction(clientX, viewport, bounds);
      const target = nearestTimelineSnapTarget(drag.snap, reached);
      drag.snap.showGuide(target);
      next = resizeRecordingSceneClip({
        clips: drag.original,
        edge: drag.edge,
        edit,
        id: drag.id,
        output: target ?? reached,
        sourceDurationMs,
      });
      seekEdge(next, drag, "move");
    }
    draftRef.current = next;
    setDraft(next);
    onDraftChange?.(next);
  };
  // The window listeners are installed once per gesture while `update` is
  // rebuilt every render, so they reach it through refs.
  const updateRef = useRef(update);
  updateRef.current = update;
  const cancelRef = useRef<() => void>(() => undefined);
  cancelRef.current = () => {
    detachRef.current();
    reset();
    onDraftChange?.(null);
  };
  useEffect(() => {
    const escape = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || !dragRef.current) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      cancelRef.current();
    };
    window.addEventListener("keydown", escape, true);
    return () => {
      window.removeEventListener("keydown", escape, true);
      detachRef.current();
    };
  }, []);

  const beginDrag = ({
    clientX,
    edge,
    id,
  }: {
    clientX: number;
    edge: RecordingSceneDragEdge;
    id: string;
  }) => {
    detachRef.current();
    const drag: Drag = {
      edge,
      id,
      moved: false,
      original: clips,
      snap: beginTimelineSnapGesture(snap, {
        mapTarget: (source) => recordingTimelineSourceToOutput(edit, source),
        threshold: 0,
      }),
      startX: clientX,
    };
    dragRef.current = drag;
    movedRef.current = false;
    if (edge !== "body") seekEdge(clips, drag, "move");
    const move = (event: PointerEvent) => {
      updateRef.current(event.clientX);
    };
    const release = (event: PointerEvent) => {
      detachRef.current();
      updateRef.current(event.clientX);
      const next = draftRef.current;
      if (next && drag.moved) {
        seekEdge(next, drag, "end");
        onCommit(next);
      }
      reset();
    };
    const abandon = () => {
      cancelRef.current();
    };
    detachRef.current = () => {
      detachRef.current = () => undefined;
      window.removeEventListener("pointermove", move, true);
      window.removeEventListener("pointerup", release, true);
      window.removeEventListener("pointercancel", abandon, true);
    };
    window.addEventListener("pointermove", move, true);
    window.addEventListener("pointerup", release, true);
    window.addEventListener("pointercancel", abandon, true);
  };

  return { beginDrag, draft, laneRef, movedRef };
}
