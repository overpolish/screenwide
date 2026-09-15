// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

import { useAnnotationDefaults } from "./annotation-defaults";
import { Annotation } from "./annotations";
import {
  mergeRecordingAnnotationClips,
  RecordingAnnotationClip,
} from "./recording-annotations";
import { RecordingTimelineEdit } from "./recording-timeline-edit";
import { RecordingVideoTrackId } from "./types";
import { useAnnotations } from "./use-annotations";

type AnnotationEvent = {
  annotations: Annotation[];
  paneIndex: number;
  selectedAnnotationId: string | null;
  sessionId: number;
  sourcePositionMs: number;
};
type AnnotationHoverEvent = { annotationId: string | null; sessionId: number };
const EMPTY_CLIPS: RecordingAnnotationClip[] = [];

export function useRecordingAnnotations({
  edit,
  isPlaying,
  onEdit,
  onSelectTrack,
  sessionId,
  sourceDurationMs,
  tool,
  trackId,
}: {
  edit: RecordingTimelineEdit;
  isPlaying: boolean;
  onEdit: (edit: RecordingTimelineEdit) => void;
  sessionId: number | null;
  sourceDurationMs: number;
  tool: string | null;
  trackId: RecordingVideoTrackId | null;
  onSelectTrack?: (track: RecordingVideoTrackId) => void;
}) {
  const clips = edit.annotationClips ?? EMPTY_CLIPS;
  const defaults = useAnnotationDefaults();
  const [previewClips, setPreviewClips] = useState<
    RecordingAnnotationClip[] | null
  >(null);
  const nativeClips = previewClips ?? clips;
  const commitClips = (next: RecordingAnnotationClip[]) => {
    if (JSON.stringify(next) !== JSON.stringify(clips))
      onEdit({ ...edit, annotationClips: next });
  };
  // Inspector edits and deletion operate on document IDs, including a selected
  // clip outside the playhead. Native gestures supply only the visible marks.
  const selection = useAnnotations({
    annotations: clips.map((clip) => clip.annotation),
    onCommit: (marks) => {
      if (isPlaying) return;
      const byId = new Map(marks.map((mark) => [mark.id, mark]));
      commitClips(
        clips.flatMap((clip) => {
          const annotation = byId.get(clip.annotation.id);
          return annotation ? [{ ...clip, annotation }] : [];
        }),
      );
    },
    workspace: "recording",
  });
  const eventRef = useRef({
    commit: (_: AnnotationEvent) => {},
    hover: selection.onHoverChange,
  });
  eventRef.current = {
    commit: (payload) => {
      const track = payload.paneIndex === 1 ? "camera" : "primary";
      commitClips(
        mergeRecordingAnnotationClips({
          annotations: payload.annotations,
          clips,
          edit,
          positionMs: payload.sourcePositionMs,
          sourceDurationMs,
          trackId: track,
        }),
      );
      selection.onSelectedChange(payload.selectedAnnotationId);
      if (payload.selectedAnnotationId) onSelectTrack?.(track);
    },
    hover: selection.onHoverChange,
  };
  useEffect(() => {
    if (sessionId === null) return;
    let disposed = false;
    const stops: (() => void)[] = [];
    const retain = (stop: () => void) => {
      if (disposed) stop();
      else stops.push(stop);
    };
    void listen<AnnotationEvent>(
      "editor://recording-annotations",
      ({ payload }) => {
        if (!disposed && payload.sessionId === sessionId)
          eventRef.current.commit(payload);
      },
    )
      .then(retain)
      .catch((cause: unknown) => {
        console.error("Could not listen for recording annotations", cause);
      });
    void listen<AnnotationHoverEvent>(
      "editor://recording-annotation-hover",
      ({ payload }) => {
        if (!disposed && payload.sessionId === sessionId)
          eventRef.current.hover(payload.annotationId);
      },
    )
      .then(retain)
      .catch((cause: unknown) => {
        console.error("Could not listen for annotation hover", cause);
      });
    return () => {
      disposed = true;
      for (const stop of stops) stop();
    };
  }, [sessionId]);
  useEffect(() => {
    if (sessionId === null) return;
    void invoke("set_recording_preview_annotations", {
      clips: nativeClips,
      defaults,
      paneIndex: trackId === null ? null : trackId === "primary" ? 0 : 1,
      selectedId: selection.selectedId,
      sessionId,
      tool: !isPlaying && (tool === "arrow" || tool === "select") ? tool : null,
    }).catch((cause: unknown) => {
      console.error("Could not update recording annotations", cause);
    });
  }, [
    nativeClips,
    defaults,
    isPlaying,
    sessionId,
    selection.selectedId,
    tool,
    trackId,
  ]);
  return {
    ...selection,
    canDelete:
      !isPlaying &&
      (selection.hasSelection ||
        ((tool === "arrow" || tool === "select") && selection.canDelete)),
    clips,
    onClipsChange: commitClips,
    onPreviewClips: setPreviewClips,
    onSelect: (id: string) => {
      const clip = clips.find((item) => item.annotation.id === id);
      if (!clip) return;
      selection.onSelectedChange(id);
      onSelectTrack?.(clip.trackId);
    },
  };
}
