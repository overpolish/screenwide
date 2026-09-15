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

import { SeekHandler } from "./scrub-timeline";
import {
  TimelineViewportState,
  timelineXToFraction,
} from "./timeline-viewport";

type Edge = "startMs" | "endMs";
type Drag = {
  edge: Edge | "body";
  id: string;
  moved: boolean;
  original: RecordingAnnotationClip[];
  startX: number;
};
export function useRecordingAnnotationDrag({
  edit,
  onPreview,
  onSeek,
  onSelect,
  sourceDurationMs,
  viewport,
}: {
  edit: RecordingTimelineEdit;
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
          clips ?? undefined,
        );
    }
    setDraft(null);
    draftRef.current = null;
    dragRef.current = null;
  };
  const cancel = () => {
    reset(false);
  };
  const finish = () => {
    reset(true);
  };
  const cancelRef = useRef(cancel);
  cancelRef.current = cancel;
  useEffect(() => {
    const abort = () => {
      cancelRef.current();
    };
    const escape = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || !dragRef.current) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      abort();
    };
    window.addEventListener("keydown", escape, true);
    window.addEventListener("blur", abort);
    return () => {
      window.removeEventListener("keydown", escape, true);
      window.removeEventListener("blur", abort);
    };
  }, []);
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
        next,
      );
  };
  return {
    cancel,
    draft,
    draftRef,
    dragRef,
    finish,
    laneRef,
    movedRef,
    setDraft,
    update,
  };
}
