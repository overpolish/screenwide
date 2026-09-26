// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

import {
  useAnnotationAngleDefault,
  useAnnotationAnimatedDefault,
  useAnnotationDefaults,
} from "./annotation-defaults";
import { Annotation, AnnotationTextEdit } from "./annotations";
import {
  clearedPinCorrections,
  isPinnable,
  pinnedClip,
  unpinnedClip,
} from "./recording-annotation-pins";
import {
  mergeRecordingAnnotationClips,
  RecordingAnnotationClip,
  RecordingAnnotationPinCommit,
  renumberedAnnotationClips,
} from "./recording-annotations";
import { RecordingTimelineEdit } from "./recording-timeline-edit";
import { drawingToolKind } from "./tool-panels/tool-registry";
import { RecordingVideoTrackId } from "./types";
import { useAnnotations } from "./use-annotations";
import { useEditorEditGesture } from "./use-editor-edit-history";
import { useRecordingPinStatus } from "./use-recording-pin-status";

type AnnotationEvent = {
  annotations: Annotation[];
  paneIndex: number;
  /** The pins of the pinned annotations among `annotations`. */
  pins: RecordingAnnotationPinCommit[];
  selectedAnnotationId: string | null;
  sessionId: number;
  sourcePositionMs: number;
  /** Where in a text box's typing this commit falls; the typing's commits
   * are grouped into one edit. */
  textEdit: AnnotationTextEdit | null;
};
type AnnotationHoverEvent = { annotationId: string | null; sessionId: number };
const EMPTY_CLIPS: RecordingAnnotationClip[] = [];

export function useRecordingAnnotations({
  edit,
  getPositionMs,
  onEdit,
  onSelectTrack,
  sessionId,
  sourceDurationMs,
  tool,
  trackId,
}: {
  edit: RecordingTimelineEdit;
  /** Where the playhead is, in source time: where a fresh pin is made. */
  getPositionMs: () => number;
  onEdit: (edit: RecordingTimelineEdit) => void;
  sessionId: number | null;
  sourceDurationMs: number;
  /** The annotation tool in hand, when one is. The native chrome learns it
   * from the layout; here it only decides what the keyboard can delete. */
  tool: import("./annotation-defaults").AnnotationTool | null;
  trackId: RecordingVideoTrackId | null;
  onSelectTrack?: (track: RecordingVideoTrackId) => void;
}) {
  const clips = edit.annotationClips ?? EMPTY_CLIPS;
  // A fresh annotation's dress and whether it animates both travel with the
  // layout the native tool draws from; animation is the annotation's own
  // property, so it rides beside the style rather than inside it. The dress is
  // the tool's own: a disc and a stroke are different measurements, so the size
  // the counter tool sends is the one counters were last drawn at.
  const defaults = useAnnotationDefaults(drawingToolKind(tool) ?? "arrow");
  const animated = useAnnotationAnimatedDefault();
  const counterAngle = useAnnotationAngleDefault();
  const editGesture = useEditorEditGesture();
  const [previewClips, setPreviewClips] = useState<
    RecordingAnnotationClip[] | null
  >(null);
  const nativeClips = previewClips ?? clips;
  const commitClips = (unnumbered: RecordingAnnotationClip[]) => {
    // Counters count what the timeline shows, so every list written here is
    // numbered by clip time: dragging one clip in front of another renumbers
    // the pair without the drag knowing about counters.
    const next = renumberedAnnotationClips(unnumbered);
    if (JSON.stringify(next) !== JSON.stringify(clips))
      onEdit({ ...edit, annotationClips: next });
  };
  const pinStatus = useRecordingPinStatus(sessionId);
  const withClip = (
    id: string,
    change: (clip: RecordingAnnotationClip) => RecordingAnnotationClip,
  ) => {
    commitClips(
      clips.map((clip) => (clip.annotation.id === id ? change(clip) : clip)),
    );
  };
  // Inspector edits and deletion operate on document IDs, including a selected
  // clip outside the playhead. Native gestures supply only the visible
  // annotations.
  const selection = useAnnotations({
    annotations: clips.map((clip) => clip.annotation),
    onCommit: (annotations) => {
      const byId = new Map(
        annotations.map((annotation) => [annotation.id, annotation]),
      );
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
      if (payload.textEdit === "begin") editGesture.beginGesture();
      const track = payload.paneIndex === 1 ? "camera" : "primary";
      commitClips(
        mergeRecordingAnnotationClips({
          annotations: payload.annotations,
          clips,
          edit,
          pins: payload.pins,
          positionMs: payload.sourcePositionMs,
          sourceDurationMs,
          trackId: track,
        }),
      );
      selection.onSelectedChange(payload.selectedAnnotationId);
      if (payload.selectedAnnotationId) onSelectTrack?.(track);
      if (payload.textEdit === "end")
        requestAnimationFrame(editGesture.endGesture);
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
      animated,
      clips: nativeClips,
      counterAngle,
      defaults,
      paneIndex: trackId === null ? null : trackId === "primary" ? 0 : 1,
      selectedId: selection.selectedId,
      sessionId,
    }).catch((cause: unknown) => {
      console.error("Could not update recording annotations", cause);
    });
  }, [
    animated,
    counterAngle,
    nativeClips,
    defaults,
    sessionId,
    selection.selectedId,
    trackId,
  ]);
  return {
    ...selection,
    canDelete: selection.hasSelection || (tool !== null && selection.canDelete),
    clips,
    onClearPinCorrections: (id: string) => {
      withClip(id, (clip) =>
        clip.pin ? { ...clip, pin: clearedPinCorrections(clip.pin) } : clip,
      );
    },
    onClipsChange: commitClips,
    onPinnedChange: (id: string, pinned: boolean) => {
      withClip(id, (clip) =>
        pinned
          ? clip.pin || !isPinnable(clip)
            ? clip
            : pinnedClip(clip, getPositionMs())
          : unpinnedClip(clip),
      );
    },
    onPreviewClips: setPreviewClips,
    onSelect: (id: string) => {
      const clip = clips.find((item) => item.annotation.id === id);
      if (!clip) return;
      selection.onSelectedChange(id);
      onSelectTrack?.(clip.trackId);
    },
    pinStatus,
  };
}
