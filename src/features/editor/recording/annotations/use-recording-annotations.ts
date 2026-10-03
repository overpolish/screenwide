// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

import {
  AnnotationTool,
  useAnnotationAngleDefault,
  useAnnotationAnimatedDefault,
  useAnnotationDefaults,
} from "../../annotations/annotation-defaults";
import { Arrangement } from "../../annotations/annotation-order";
import { AnnotationFrame, pacedClip } from "../../annotations/annotation-pace";
import { Annotation, AnnotationTextEdit } from "../../annotations/annotations";
import { useAnnotations } from "../../annotations/use-annotations";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { drawingToolKind } from "../../tool-panels/tool-registry";
import { CameraOverlaySettings, RecordingVideoTrackId } from "../../types";
import { useEditorEditGesture } from "../../use-editor-edit-history";

import {
  arrangedRecordingAnnotations,
  recordingCameraPlacement,
} from "./recording-annotation-layers";
import { recordingAnnotationPinning } from "./recording-annotation-pinning";
import {
  mergeRecordingAnnotationClips,
  RecordingAnnotationClip,
  RecordingAnnotationPinCommit,
  renumberedAnnotationClips,
} from "./recording-annotations";
import { useRecordingPinStatus } from "./use-recording-pin-status";

type AnnotationEvent = {
  annotations: Annotation[];
  paneIndex: number;
  /** The pins of the pinned annotations among `annotations`. */
  pins: RecordingAnnotationPinCommit[];
  /** Every annotation chosen once the commit lands. */
  selectedAnnotationIds: string[];
  sessionId: number;
  sourcePositionMs: number;
  /** Where in a text box's typing this commit falls; the typing's commits
   * are grouped into one edit. */
  textEdit: AnnotationTextEdit | null;
};
type AnnotationHoverEvent = { annotationId: string | null; sessionId: number };
const EMPTY_CLIPS: RecordingAnnotationClip[] = [];

export function useRecordingAnnotations({
  bakedLayers,
  edit,
  frames,
  getPositionMs,
  onEdit,
  onSelectTrack,
  screenCaptureScale,
  sessionId,
  sourceDurationMs,
  tool,
  trackId,
}: {
  /** The screen's output and the camera's placement on it at the playhead,
   * where One video draws the camera into the screen's video; null where it
   * does not, and annotations cannot move between the two. */
  bakedLayers: {
    cameraOverlay: CameraOverlaySettings;
    screen: ScreenshotOutputSettings;
  } | null;
  edit: RecordingTimelineEdit;
  /** Each pane's picture, in its own source pixels: what an annotation's
   * path is paced against. */
  frames: Partial<Record<RecordingVideoTrackId, AnnotationFrame>>;
  /** Where the playhead is, in source time: where a fresh pin is made. */
  getPositionMs: () => number;
  onEdit: (edit: RecordingTimelineEdit) => void;
  /** Captured pixels per logical point of the screen. */
  screenCaptureScale: number;
  sessionId: number | null;
  sourceDurationMs: number;
  /** The annotation tool in hand, when one is. The native chrome learns it
   * from the layout; here it only decides what the keyboard can delete. */
  tool: AnnotationTool | null;
  trackId: RecordingVideoTrackId | null;
  onSelectTrack?: (track: RecordingVideoTrackId) => void;
}) {
  const clips = edit.annotationClips ?? EMPTY_CLIPS;
  const cameraPlacement =
    bakedLayers &&
    recordingCameraPlacement({
      ...bakedLayers,
      cameraSource: frames.camera,
      screenCaptureScale,
      screenSource: frames.primary,
    });
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
    // numbered by clip time: dragging one clip in front of another in time
    // renumbers the pair, while reordering the drawing order does not. Every
    // clip is paced by its annotation as it now stands, so a path drawn
    // longer takes longer to draw in.
    const next = renumberedAnnotationClips(unnumbered, clips).map((clip) =>
      pacedClip(clip, frames[clip.trackId]),
    );
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
  // annotations. A pinned clip follows its content on its own, so it is never
  // chosen with others.
  const pinned = new Set(
    clips.flatMap((clip) => (clip.pin ? [clip.annotation.id] : [])),
  );
  const selection = useAnnotations({
    annotations: clips.map((clip) => clip.annotation),
    groupable: (id) => !pinned.has(id),
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
          frame: frames[track],
          pins: payload.pins,
          positionMs: payload.sourcePositionMs,
          sourceDurationMs,
          trackId: track,
        }),
      );
      selection.onSelectedIdsChange(payload.selectedAnnotationIds);
      if (payload.selectedAnnotationIds.length > 0) onSelectTrack?.(track);
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
      selectedIds: [...selection.selectedIds],
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
    selection.selectedIds,
    trackId,
  ]);
  const pinning = recordingAnnotationPinning({
    clips,
    getPositionMs,
    pinStatus,
    sessionId,
    withClip,
  });
  return {
    ...selection,
    /** Moves the annotations in hand past the clips showing with them,
     * several together keeping their own stacking, answering whether there
     * were any. A step past the end of their track hands them to the other
     * picture, which is then the one in hand. */
    arrangeSelected: (arrangement: Arrangement) => {
      if (selection.selectedIds.size === 0) return false;
      const isMember = (clip: RecordingAnnotationClip) =>
        selection.selectedIds.has(clip.annotation.id);
      const next = arrangedRecordingAnnotations({
        arrangement,
        clips,
        isMember,
        placement: cameraPlacement,
      });
      if (next === clips) return true;
      commitClips(next);
      const moved = next.find(isMember);
      if (moved && moved.trackId !== clips.find(isMember)?.trackId)
        onSelectTrack?.(moved.trackId);
      return true;
    },
    /** Where One video draws the screen and the camera at the playhead, which
     * a move between the two keeps an annotation's place against. */
    cameraPlacement,
    canDelete: selection.hasSelection || (tool !== null && selection.canDelete),
    clips,
    onClipsChange: commitClips,
    onPreviewClips: setPreviewClips,
    /** Choose the clip named `id` alone, or with `toggle`, add it to the
     * choice or take it away. Its pane becomes the one in hand. */
    onSelect: (id: string, toggle: boolean) => {
      const clip = clips.find((item) => item.annotation.id === id);
      if (!clip) return;
      selection.selectAnnotation(id, toggle);
      onSelectTrack?.(clip.trackId);
    },
    /** Choose the clips a band swept over, alone or added to the choice. */
    onSelectSwept: selection.selectAnnotations,
    pinStatus,
    pinning,
  };
}
