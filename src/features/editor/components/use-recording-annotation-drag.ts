// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { resizeRecordingAnnotationClip } from "../recording-annotation-geometry";
import {
  RecordingAnnotationClip,
  renumberedAnnotationClips,
} from "../recording-annotations";
import {
  RecordingTimelineEdit,
  recordingTimelineSourceToOutput,
} from "../recording-timeline-edit";

import {
  carriedAxes,
  carriedClips,
  previewedWhole,
} from "./recording-annotation-drag-draft";
import { SeekHandler } from "./timeline-seek";
import {
  beginTimelineSnapGesture,
  nearestTimelineSnapTarget,
  TIMELINE_SNAP_THRESHOLD_PX,
  timelineSnapRangeShift,
  TimelineSnapGesture,
  useTimelineSnap,
} from "./timeline-snap";
import {
  TimelineViewportState,
  timelineXToFraction,
} from "./timeline-viewport";

type Edge = "startMs" | "endMs";
type Drag = {
  edge: Edge | "body";
  id: string;
  /** Where the pointer last was, for a Shift press to resample. */
  last: { x: number; y: number };
  moved: boolean;
  original: RecordingAnnotationClip[];
  snap: TimelineSnapGesture;
  startX: number;
  startY: number;
};

export function useRecordingAnnotationDrag({
  clips,
  edit,
  onCommit,
  onPreview,
  onSeek,
  onSelect,
  sourceDurationMs,
  viewport,
}: {
  clips: RecordingAnnotationClip[];
  edit: RecordingTimelineEdit;
  onCommit: (clips: RecordingAnnotationClip[]) => void;
  onSelect: (id: string) => void;
  sourceDurationMs: number;
  viewport: TimelineViewportState;
  onPreview?: (clips: RecordingAnnotationClip[] | null) => void;
  onSeek?: SeekHandler;
}) {
  const [draft, setDraft] = useState<RecordingAnnotationClip[] | null>(null);
  const draftRef = useRef<RecordingAnnotationClip[] | null>(null);
  const dragRef = useRef<Drag | null>(null);
  const movedRef = useRef(false);
  const laneRef = useRef<HTMLDivElement>(null);
  const snap = useTimelineSnap();
  const detachRef = useRef<() => void>(() => undefined);
  // The window listeners are installed once per gesture while the sample
  // handler below is rebuilt every render, so they reach it through a ref
  // rather than closing over the render that started the drag.
  const updateRef = useRef<
    (clientX: number, clientY: number, shiftKey: boolean) => void
  >(() => undefined);
  // A carried clip is previewed as it will play, reveal and all, so the
  // frame under the playhead follows it through time. A trim seeks the
  // preview itself, drawing the annotation whole at the handle.
  useEffect(() => {
    onPreview?.(dragRef.current?.edge === "body" ? draft : null);
  }, [draft, onPreview]);
  useEffect(
    () => () => {
      onPreview?.(null);
    },
    [onPreview],
  );
  const reset = (commit: boolean) => {
    const drag = dragRef.current;
    if (drag?.edge !== "body") {
      const clips = commit ? draftRef.current : drag?.original;
      const clip = clips?.find((item) => item.annotation.id === drag?.id);
      if (clip && drag && sourceDurationMs > 0)
        onSeek?.(
          recordingTimelineSourceToOutput(
            edit,
            (drag.edge === "endMs" ? clip.endMs - 1 : clip.startMs) /
              sourceDurationMs,
          ),
          "end",
          // The gesture is over, so the seek that settles it shows the
          // annotation at the reveal its own bounds put it at once more.
          clips ?? undefined,
        );
    }
    drag?.snap.showGuide(null);
    setDraft(null);
    draftRef.current = null;
    dragRef.current = null;
  };
  const cancel = () => {
    detachRef.current();
    reset(false);
  };
  const cancelRef = useRef(cancel);
  cancelRef.current = cancel;
  useEffect(() => {
    // Escape abandons a drag in flight. A window blur does not: pushing
    // preview frames hands focus to the native surface the frames are drawn
    // on, and a pointer that is still down is still a drag.
    const escape = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || !dragRef.current) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      cancelRef.current();
    };
    window.addEventListener("keydown", escape, true);
    return () => {
      window.removeEventListener("keydown", escape, true);
    };
  }, []);

  /**
   * Takes a press on a clip or one of its edges and owns the gesture until
   * the pointer is released. A clip's body carried sideways moves it in time
   * and carried up or down moves it through the drawing order, a row at a
   * time.
   *
   * The moves and the release are listened for on the window rather than on
   * the element pressed. The lane restacks its clips as they are dragged -
   * two annotations that come to overlap each need a row - so the element under
   * the pointer is moved in the DOM part way through the gesture, and a
   * gesture that depended on that element keeping pointer capture lost the
   * drag exactly when it started to matter.
   */
  const beginDrag = ({
    clientX,
    clientY,
    edge,
    id,
  }: {
    clientX: number;
    clientY: number;
    edge: Edge | "body";
    id: string;
  }) => {
    detachRef.current();
    dragRef.current = {
      edge,
      id,
      last: { x: clientX, y: clientY },
      moved: false,
      original: clips,
      snap: beginTimelineSnapGesture(snap, {
        excludeAnnotationId: id,
        mapTarget: (source) => recordingTimelineSourceToOutput(edit, source),
        threshold: 0,
      }),
      startX: clientX,
      startY: clientY,
    };
    draftRef.current = clips;
    movedRef.current = false;
    setDraft(clips);
    const move = (event: PointerEvent) => {
      updateRef.current(event.clientX, event.clientY, event.shiftKey);
    };
    // Shift pressed or let go with the pointer still takes effect at once.
    const shift = (event: KeyboardEvent) => {
      const last = dragRef.current?.last;
      if (event.key === "Shift" && last)
        updateRef.current(last.x, last.y, event.shiftKey);
    };
    const release = (event: PointerEvent) => {
      detachRef.current();
      updateRef.current(event.clientX, event.clientY, event.shiftKey);
      const next = draftRef.current;
      const moved = movedRef.current;
      reset(true);
      if (next && (moved || edge !== "body")) onCommit(next);
    };
    const abandon = () => {
      cancelRef.current();
    };
    detachRef.current = () => {
      detachRef.current = () => undefined;
      window.removeEventListener("pointermove", move, true);
      window.removeEventListener("pointerup", release, true);
      window.removeEventListener("pointercancel", abandon, true);
      window.removeEventListener("keydown", shift, true);
      window.removeEventListener("keyup", shift, true);
    };
    window.addEventListener("pointermove", move, true);
    window.addEventListener("pointerup", release, true);
    window.addEventListener("pointercancel", abandon, true);
    window.addEventListener("keydown", shift, true);
    window.addEventListener("keyup", shift, true);
  };
  const update = (clientX: number, clientY: number, shiftKey: boolean) => {
    const drag = dragRef.current;
    const bounds = laneRef.current?.getBoundingClientRect();
    if (!drag || !bounds) return;
    drag.last = { x: clientX, y: clientY };
    const travelled =
      Math.abs(clientX - drag.startX) >= 4 ||
      (drag.edge === "body" && Math.abs(clientY - drag.startY) >= 4);
    if (!drag.moved && !travelled) return;
    if (!drag.moved && drag.edge === "body") onSelect(drag.id);
    drag.moved = true;
    movedRef.current = true;
    // Retaken every move: a wheel zoom mid-drag changes what 8px spans.
    drag.snap.threshold =
      TIMELINE_SNAP_THRESHOLD_PX / (viewport.zoom * bounds.width);
    if (drag.edge === "body") {
      const clip = drag.original.find((item) => item.annotation.id === drag.id);
      if (!clip || sourceDurationMs <= 0) return;
      const start = recordingTimelineSourceToOutput(
        edit,
        clip.startMs / sourceDurationMs,
      );
      const end = recordingTimelineSourceToOutput(
        edit,
        clip.endMs / sourceDurationMs,
      );
      const axes = carriedAxes(
        clientX - drag.startX,
        clientY - drag.startY,
        shiftKey,
      );
      const shift = axes.time
        ? (clientX - drag.startX) / (viewport.zoom * bounds.width)
        : 0;
      const snapped = axes.time
        ? timelineSnapRangeShift(drag.snap, start + shift, end + shift)
        : null;
      drag.snap.showGuide(snapped?.target ?? null);
      const next = carriedClips({
        deltaOutput: shift + (snapped?.shift ?? 0),
        edit,
        id: drag.id,
        lift: axes.order ? drag.startY - clientY : 0,
        original: drag.original,
        sourceDurationMs,
      });
      draftRef.current = next;
      setDraft(next);
      return;
    }
    const reached = timelineXToFraction(clientX, viewport, bounds);
    const target = nearestTimelineSnapTarget(drag.snap, reached);
    drag.snap.showGuide(target);
    const next = renumberedAnnotationClips(
      resizeRecordingAnnotationClip({
        clips: drag.original,
        edge: drag.edge,
        edit,
        id: drag.id,
        output: target ?? reached,
        sourceDurationMs,
      }),
      drag.original,
    );
    draftRef.current = next;
    setDraft(next);
    const edgeClip = next.find((item) => item.annotation.id === drag.id);
    if (edgeClip && sourceDurationMs > 0)
      onSeek?.(
        recordingTimelineSourceToOutput(
          edit,
          (drag.edge === "endMs" ? edgeClip.endMs - 1 : edgeClip.startMs) /
            sourceDurationMs,
        ),
        "move",
        previewedWhole(next, drag.id),
      );
  };
  updateRef.current = update;
  return { beginDrag, cancel, draft, laneRef, movedRef };
}
