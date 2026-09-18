// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import {
  moveRecordingAnnotationClip,
  resizeRecordingAnnotationClip,
} from "../recording-annotation-geometry";
import { RecordingAnnotationClip } from "../recording-annotations";
import {
  RecordingTimelineEdit,
  recordingTimelineSourceToOutput,
} from "../recording-timeline-edit";

import { SeekHandler } from "./timeline-seek";
import {
  TimelineViewportState,
  timelineXToFraction,
} from "./timeline-viewport";

type Edge = "startMs" | "endMs";

/**
 * The clips as the preview should show them while `id` is being dragged: that
 * annotation drawn whole rather than at the reveal its clip's bounds put it at.
 *
 * A trim handle sits exactly where the annotation is arriving or leaving, so
 * the frame it seeks to is the one frame where the annotation is barely there -
 * which is no use at all for deciding where the handle belongs. Marking the
 * clip as not animated is how an annotation is drawn whole everywhere else, and
 * it is the preview's own copy: the list that reaches the document keeps its
 * animation.
 */
export const previewedWhole = (
  clips: RecordingAnnotationClip[],
  id: string,
): RecordingAnnotationClip[] =>
  clips.map((clip) =>
    clip.annotation.id === id && clip.annotation.animated
      ? { ...clip, annotation: { ...clip.annotation, animated: false } }
      : clip,
  );
type Drag = {
  edge: Edge | "body";
  id: string;
  moved: boolean;
  original: RecordingAnnotationClip[];
  startX: number;
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
  const detachRef = useRef<() => void>(() => undefined);
  // The window listeners are installed once per gesture while the sample
  // handler below is rebuilt every render, so they reach it through a ref
  // rather than closing over the render that started the drag.
  const updateRef = useRef<(clientX: number) => void>(() => undefined);
  useEffect(() => {
    const drag = dragRef.current;
    onPreview?.(
      drag?.edge === "body" && draft ? previewedWhole(draft, drag.id) : null,
    );
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
   * the pointer is released.
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
    edge,
    id,
  }: {
    clientX: number;
    edge: Edge | "body";
    id: string;
  }) => {
    detachRef.current();
    dragRef.current = {
      edge,
      id,
      moved: false,
      original: clips,
      startX: clientX,
    };
    draftRef.current = clips;
    movedRef.current = false;
    setDraft(clips);
    const move = (event: PointerEvent) => {
      updateRef.current(event.clientX);
    };
    const release = (event: PointerEvent) => {
      detachRef.current();
      updateRef.current(event.clientX);
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
    };
    window.addEventListener("pointermove", move, true);
    window.addEventListener("pointerup", release, true);
    window.addEventListener("pointercancel", abandon, true);
  };
  const update = (clientX: number) => {
    const drag = dragRef.current;
    const bounds = laneRef.current?.getBoundingClientRect();
    if (!drag || !bounds) return;
    if (!drag.moved && Math.abs(clientX - drag.startX) < 4) return;
    if (!drag.moved && drag.edge === "body") onSelect(drag.id);
    drag.moved = true;
    movedRef.current = true;
    if (drag.edge === "body") {
      const clip = drag.original.find((item) => item.annotation.id === drag.id);
      if (!clip) return;
      const next = drag.original.map((item) =>
        item.annotation.id === drag.id
          ? moveRecordingAnnotationClip({
              clip,
              deltaOutput:
                (clientX - drag.startX) / (viewport.zoom * bounds.width),
              edit,
              sourceDurationMs,
            })
          : item,
      );
      draftRef.current = next;
      setDraft(next);
      return;
    }
    const next = resizeRecordingAnnotationClip({
      clips: drag.original,
      edge: drag.edge,
      edit,
      id: drag.id,
      output: timelineXToFraction(clientX, viewport, bounds),
      sourceDurationMs,
    });
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
