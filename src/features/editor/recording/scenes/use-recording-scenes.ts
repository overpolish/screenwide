// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useReducer, useRef } from "react";

import { type SceneDrawing } from "../../glyphs/scene-layout-drawing";
import { screenshotLayout } from "../../screenshot/screenshot-layout";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import {
  createRecordingTimelineEdit,
  RecordingTimelineEdit,
} from "../../timeline/editing/recording-timeline-edit";
import {
  recordingTimelineLengthMs,
  recordingTimelineMsAt,
} from "../../timeline/editing/recording-timeline-time";
import { Playhead } from "../../timeline/scrub-playhead";
import { timedSpanClipAt } from "../../timeline/tracks/timed-span-clips";
import {
  TimedSpanSelection,
  useTimedSpanSelection,
} from "../../timeline/tracks/use-timed-span-selection";
import { CameraOverlaySettings, RecordingVideoTrackId } from "../../types";
import { withSceneClips } from "../annotations/recording-annotation-scenes";
import { retimedSpanClips } from "../recording-overlay-retime";

import {
  ownEditedAutoZooms,
  planRecordingAutoZoom,
  visibleScreenArea,
  withAutoZooms,
} from "./recording-scene-auto-zoom";
import {
  RecordingSceneControls,
  usePublishRecordingScene,
} from "./recording-scene-channel";
import { sceneClipDrawing } from "./recording-scene-drawing";
import { recordingSceneEdits } from "./recording-scene-edits";
import {
  scenePaneFraming,
  selectedPaneFraming,
} from "./recording-scene-pane-framing";
import { toolPanelSceneOf } from "./recording-scene-panel-snapshot";
import { recenterSceneScreen } from "./recording-scene-recenter";
import { RecordingSceneClip } from "./recording-scenes";
import { useScenePreviewSync } from "./use-scene-preview-sync";

const EMPTY_CLIPS: RecordingSceneClip[] = [];

export type RecordingScenes = {
  /** Whether the scenes that place a camera can play: there is a camera, and
   * it is baked into the picture. Without it they stand idle, kept but
   * neither shown nor exported, while full scenes still play. */
  canPlaceCamera: boolean;
  clips: RecordingSceneClip[];
  /** The edits the Scene panel makes, which the preview's own menus and keys
   * reach as well. */
  controls: RecordingSceneControls;
  /** The scene under the playhead, null between scenes. */
  current: RecordingSceneClip | null;
  /** How a scene's layout is drawn, as the Scene island draws it. */
  drawingOf: (clip: RecordingSceneClip) => SceneDrawing;
  /** Whether the recording has a screen to arrange. */
  isAvailable: boolean;
  onClipsChange: (clips: RecordingSceneClip[]) => void;
  /** Shows a lane drag's draft in the preview while it lasts, or the
   * committed clips again for null. */
  onDraftChange: (clips: RecordingSceneClip[] | null) => void;
  /** The scenes chosen in their lane. */
  selection: TimedSpanSelection;
};

/**
 * The recording's scene clips and the edits the lane and the Scene panel
 * make to them. The panel acts on whichever scene is under the playhead, and
 * adds one there where there is none; the scenes chosen in the lane are what
 * Delete takes away. Every change is a timeline edit, so it is saved with the
 * recording and undone with the rest of the timeline.
 */
export function useRecordingScenes({
  artifactId,
  cameraOverlay,
  edit,
  frames,
  getPositionMs,
  isBaked,
  isPlaying,
  onEdit,
  output,
  playhead,
  positionMs,
  selectsCamera,
  sessionId,
  sourceDurationMs,
}: {
  artifactId: number;
  /** Where the recording's own composition puts the camera. */
  cameraOverlay: CameraOverlaySettings;
  edit: RecordingTimelineEdit;
  /** Each pane's picture size: a scene needs a screen, and a camera to place
   * one. */
  frames: Partial<
    Record<RecordingVideoTrackId, { height: number; width: number }>
  >;
  /** Where the playhead is now, in source time, for an edit the panel asks
   * for while the recording plays. */
  getPositionMs: () => number;
  isBaked: boolean;
  isPlaying: boolean;
  onEdit: (edit: RecordingTimelineEdit) => void;
  /** The screen's composition: the finished frame's size, and the visible
   * screen inside it, which the panel's drawings are made at. */
  output: ScreenshotOutputSettings;
  /** Ticks as the playhead moves, which the panel follows while it plays. */
  playhead: Pick<Playhead, "subscribe">;
  /** Where the playhead last settled, in source time: what the panel shows
   * while the recording is paused. */
  positionMs: number;
  /** Whether the camera is the selected track, which points the panel's zoom
   * and position at it where the scene places it. */
  selectsCamera: boolean;
  /** The native preview session that draws the scenes, once it is open. */
  sessionId: number | null;
  sourceDurationMs: number;
}): RecordingScenes {
  const isAvailable = Boolean(frames.primary);
  const hasCamera = Boolean(frames.camera);
  // The camera's own shape, which the pairs keep it at.
  const cameraAspect = frames.camera
    ? frames.camera.width / Math.max(1, frames.camera.height)
    : 16 / 9;
  // The camera a scene can place: one drawn into the picture.
  const sceneCamera =
    frames.camera && isBaked
      ? { aspect: cameraAspect, overlay: cameraOverlay }
      : null;
  const clips = edit.sceneClips ?? EMPTY_CLIPS;
  // A plan arrives after the edit it was asked from may have moved on, so it
  // lands on the latest one.
  const editRef = useRef(edit);
  editRef.current = edit;
  // A press that changes nothing, a drag let go where it began, must leave
  // nothing to undo. A scene taken away takes annotations' places in it.
  const commitClips = (next: RecordingSceneClip[]) => {
    const latest = editRef.current;
    if (
      JSON.stringify(next) !== JSON.stringify(latest.sceneClips ?? EMPTY_CLIPS)
    )
      onEdit(withSceneClips(latest, next));
  };
  const commit = (next: RecordingSceneClip[]) => {
    commitClips(ownEditedAutoZooms(clips, next));
  };
  // Scenes are timed on the edited video; the playhead is read in the
  // recording.
  const timelineMsAt = (sourceMs: number) =>
    recordingTimelineMsAt(edit, sourceDurationMs, sourceMs);
  // The panel follows the scene under the playhead. While the recording plays
  // the settled position stands still, so the playhead itself is read, and
  // its ticks only redraw the panel as it crosses a clip's edge; reading it
  // from the first render keeps the panel from showing no scene until the
  // first tick.
  const [, redraw] = useReducer((count: number) => count + 1, 0);
  const playingClipRef = useRef<string | null>(null);
  useEffect(() => {
    if (!isPlaying) return;
    return playhead.subscribe(() => {
      const id =
        timedSpanClipAt(
          clips,
          recordingTimelineMsAt(edit, sourceDurationMs, getPositionMs()),
        )?.id ?? null;
      if (id === playingClipRef.current) return;
      playingClipRef.current = id;
      redraw();
    });
  }, [clips, edit, getPositionMs, isPlaying, playhead, sourceDurationMs]);
  const current = timedSpanClipAt(
    clips,
    timelineMsAt(isPlaying ? getPositionMs() : positionMs),
  );
  // The pane named, as the Scene island frames either; else the selected
  // one, as the picture's drag reaches it.
  const paneOf = (clip: RecordingSceneClip, pane?: "camera" | "screen") =>
    pane
      ? scenePaneFraming({ camera: sceneCamera, clip, output, pane })
      : selectedPaneFraming({
          camera: sceneCamera,
          clip,
          output,
          selectsCamera,
        });
  const currentPane = current ? paneOf(current) : null;
  const showDraft = useScenePreviewSync({
    clips,
    edit,
    look: { cameraOverlay, output },
    sessionId,
    sourceDurationMs,
  });
  const selection = useTimedSpanSelection({
    clips,
    commit,
    currentId: current?.id ?? null,
  });
  const controls: RecordingSceneControls = {
    ...recordingSceneEdits({
      camera: sceneCamera,
      clips,
      commit,
      getTimelineMs: () => timelineMsAt(getPositionMs()),
      lengthMs: recordingTimelineLengthMs(edit, sourceDurationMs),
      output,
      paneOf,
    }),
    autoZoom: () => {
      planRecordingAutoZoom(
        artifactId,
        visibleScreenArea(output, frames.primary),
      )
        .then((planned) => {
          // The plan is timed on the whole recording; it lands on the
          // timeline as it now stands.
          const latest = editRef.current;
          commitClips(
            withAutoZooms(
              latest.sceneClips ?? EMPTY_CLIPS,
              retimedSpanClips(planned, {
                next: latest.segments,
                previous: createRecordingTimelineEdit(latest.artifactId)
                  .segments,
                sourceDurationMs,
              }),
            ),
          );
        })
        .catch((cause: unknown) => {
          console.error("Could not make the auto zooms", cause);
        });
    },
    clearAutoZooms: () => {
      commitClips(withAutoZooms(editRef.current.sceneClips ?? EMPTY_CLIPS, []));
    },
    recenterScreen: () => {
      const positionMs = getPositionMs();
      const target = timedSpanClipAt(clips, timelineMsAt(positionMs));
      if (!target || !frames.primary) return;
      recenterSceneScreen({
        artifactId,
        camera: sceneCamera,
        clipId: target.id,
        commit: (next) => {
          const latest = editRef.current.sceneClips ?? EMPTY_CLIPS;
          commitClips(ownEditedAutoZooms(latest, next));
        },
        latest: () => editRef.current.sceneClips ?? EMPTY_CLIPS,
        layout: screenshotLayout(frames.primary, output),
        output,
        positionMs,
        source: frames.primary,
      });
    },
  };
  usePublishRecordingScene(
    "recording",
    isAvailable
      ? toolPanelSceneOf({
          camera: sceneCamera,
          cameraAspect,
          cameraOverlay,
          clips,
          current,
          currentPane,
          hasCamera,
          isBaked,
          output,
        })
      : null,
    controls,
  );
  return {
    canPlaceCamera: hasCamera && isBaked,
    clips,
    controls,
    current,
    drawingOf: (clip) =>
      sceneClipDrawing(clip, {
        camera: sceneCamera,
        cameraAspect,
        cameraOverlay,
        output,
      }),
    isAvailable,
    onClipsChange: commit,
    onDraftChange: showDraft,
    selection,
  };
}
