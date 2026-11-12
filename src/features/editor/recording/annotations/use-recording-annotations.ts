// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { AnnotationTool } from "../../annotations/annotation-defaults";
import { Arrangement } from "../../annotations/annotation-order";
import { AnnotationFrame, pacedClip } from "../../annotations/annotation-pace";
import { useAnnotations } from "../../annotations/use-annotations";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { recordingTimelineLengthMs } from "../../timeline/editing/recording-timeline-time";
import { type Playhead } from "../../timeline/scrub-playhead";
import { CameraOverlaySettings, RecordingVideoTrackId } from "../../types";
import { useEditorEditGesture } from "../../use-editor-edit-history";

import { heldLengthClip } from "./image-once";
import {
  arrangedRecordingAnnotations,
  recordingCameraPlacement,
} from "./recording-annotation-layers";
import { recordingAnnotationPinning } from "./recording-annotation-pinning";
import {
  annotationInScene,
  isPlacedInScene,
  withEditInScene,
  withoutSceneShape,
} from "./recording-annotation-scenes";
import {
  laneAnnotationClips,
  useHiddenAnnotationsLetGo,
  withShownAnnotations,
} from "./recording-annotation-visibility";
import {
  mergeRecordingAnnotationClips,
  RecordingAnnotationClip,
  renumberedAnnotationClips,
} from "./recording-annotations";
import { withRecordingSpotlightBlurShared } from "./recording-spotlight-blur";
import { useRecordingAnnotationEvents } from "./use-recording-annotation-events";
import { useRecordingAnnotationPins } from "./use-recording-annotation-pins";
import { useRecordingAnnotationSync } from "./use-recording-annotation-sync";
import { useRecordingPinStatus } from "./use-recording-pin-status";

const EMPTY_CLIPS: RecordingAnnotationClip[] = [];

export function useRecordingAnnotations({
  bakedLayers,
  edit,
  frames,
  getPositionMs,
  onChoiceDeleted,
  onEdit,
  onSelectTrack,
  playhead,
  sceneId = null,
  screenCaptureScale,
  sessionId,
  shownTracks,
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
  /** The playhead, heard for how a pin stands where it is. */
  playhead: Playhead;
  /** Captured pixels per logical point of the screen. */
  screenCaptureScale: number;
  sessionId: number | null;
  /** The video tracks turned on, whose annotations the lane shows. */
  shownTracks: ReadonlySet<RecordingVideoTrackId>;
  sourceDurationMs: number;
  /** The annotation tool in hand, when one is. The native chrome learns it
   * from the layout; here it only decides what the keyboard can delete. */
  tool: AnnotationTool | null;
  trackId: RecordingVideoTrackId | null;
  onChoiceDeleted?: () => void;
  onSelectTrack?: (track: RecordingVideoTrackId) => void;
  /** The scene the playhead is in, if any: where an annotation is chosen
   * as it stands there, and an edit that moves it is that scene's. */
  sceneId?: string | null;
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
    // longer takes longer to draw in, and an image played once lasts one
    // run of its animation, whatever edit asked for another length.
    // Spotlights shown together share one Blur setting, whichever edit
    // brings them together.
    const next = renumberedAnnotationClips(
      withRecordingSpotlightBlurShared(unnumbered, clips),
      clips,
    ).map((clip) =>
      pacedClip(
        heldLengthClip(clip, recordingTimelineLengthMs(edit, sourceDurationMs)),
        frames[clip.trackId],
      ),
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
  // A track turned off keeps its annotations out of the lane and the choice;
  // an edit made without them lays them back where they were.
  const lane = laneAnnotationClips(clips, shownTracks, {
    commit: commitClips,
    preview: setPreviewClips,
  });
  const pinning = recordingAnnotationPinning({
    clips,
    edit,
    getPositionMs,
    pinStatus,
    sessionId,
    sourceDurationMs,
    withClip,
  });
  const pins = useRecordingAnnotationPins({
    clips,
    getPositionMs,
    pinStatus,
    pinning,
    playhead,
  });
  // At the playhead in a scene, each annotation is chosen and dressed as it
  // stands there, and an edit that moves it is that scene's.
  const scenes = sceneId
    ? {
        isPlaced: (id: string) =>
          clips.some(
            (clip) =>
              clip.annotation.id === id &&
              !clip.pin &&
              isPlacedInScene(clip, sceneId),
          ),
        reset: (id: string) => {
          withClip(id, (clip) => withoutSceneShape(clip, sceneId));
        },
      }
    : undefined;
  const selection = useAnnotations({
    annotations: lane.clips.map((clip) => annotationInScene(clip, sceneId)),
    groupable: (id) => !pinned.has(id),
    onChoiceDeleted,
    onCommit: (annotations) => {
      const shown = withShownAnnotations(clips, annotations, shownTracks);
      commitClips(
        sceneId
          ? shown.map((clip) => {
              const before = clips.find(
                (each) => each.annotation.id === clip.annotation.id,
              );
              return before && !before.pin
                ? withEditInScene(before, clip.annotation, sceneId)
                : clip;
            })
          : shown,
      );
    },
    pins,
    scenes,
    workspace: "recording",
  });
  useHiddenAnnotationsLetGo(shownTracks, selection);
  useRecordingAnnotationEvents(sessionId, {
    onCommit: (payload) => {
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
          sceneShapes: payload.sceneShapes,
          sourceDurationMs,
          trackId: track,
        }),
      );
      selection.onSelectedIdsChange(payload.selectedAnnotationIds);
      if (payload.selectedAnnotationIds.length > 0) onSelectTrack?.(track);
      if (payload.textEdit === "end")
        requestAnimationFrame(editGesture.endGesture);
    },
    onHover: selection.onHoverChange,
  });
  useRecordingAnnotationSync({
    clips: nativeClips,
    edit,
    selectedIds: selection.selectedIds,
    sessionId,
    sourceDurationMs,
    tool,
    trackId,
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
    ...lane,
    /** Choose the clip named `id` alone, or with `toggle`, add it to the
     * choice or take it away. Its pane becomes the one in hand. */
    onSelect: (id: string, toggle: boolean) => {
      const clip = lane.clips.find((item) => item.annotation.id === id);
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
