// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useReducer, useRef } from "react";

import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { Playhead } from "../../timeline/scrub-playhead";
import { CameraOverlaySettings, RecordingVideoTrackId } from "../../types";

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
import { recordingSceneEdits } from "./recording-scene-edits";
import { drawnCameraFrame } from "./recording-scene-framing";
import { selectedPaneFraming } from "./recording-scene-pane-framing";
import { RecordingSceneClip, recordingSceneClipAt } from "./recording-scenes";
import {
  RecordingSceneSelection,
  useRecordingSceneSelection,
} from "./use-recording-scene-selection";
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
  /** Whether the recording has a screen to arrange. */
  isAvailable: boolean;
  onClipsChange: (clips: RecordingSceneClip[]) => void;
  /** Shows a lane drag's draft in the preview while it lasts, or the
   * committed clips again for null. */
  onDraftChange: (clips: RecordingSceneClip[] | null) => void;
  /** The scenes chosen in their lane. */
  selection: RecordingSceneSelection;
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
  // nothing to undo.
  const commitClips = (next: RecordingSceneClip[]) => {
    const latest = editRef.current;
    if (
      JSON.stringify(next) !== JSON.stringify(latest.sceneClips ?? EMPTY_CLIPS)
    )
      onEdit({ ...latest, sceneClips: next });
  };
  const commit = (next: RecordingSceneClip[]) => {
    commitClips(ownEditedAutoZooms(clips, next));
  };
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
      const id = recordingSceneClipAt(clips, getPositionMs())?.id ?? null;
      if (id === playingClipRef.current) return;
      playingClipRef.current = id;
      redraw();
    });
  }, [clips, getPositionMs, isPlaying, playhead]);
  const current = recordingSceneClipAt(
    clips,
    isPlaying ? getPositionMs() : positionMs,
  );
  const paneOf = (clip: RecordingSceneClip) =>
    selectedPaneFraming({ camera: sceneCamera, clip, output, selectsCamera });
  const currentPane = current ? paneOf(current) : null;
  const showDraft = useScenePreviewSync({ clips, sessionId });
  const selection = useRecordingSceneSelection({
    clips,
    commit,
    currentId: current?.id ?? null,
  });
  // A composition from before placement was measured in output pixels has no
  // crop; the whole frame is its screen.
  const crop =
    output.cropWidth > 0 && output.cropHeight > 0
      ? {
          height: output.cropHeight,
          width: output.cropWidth,
          x: output.cropX,
          y: output.cropY,
        }
      : { height: output.height, width: output.width, x: 0, y: 0 };
  const screenAspect = crop.width / crop.height;
  const share = (rect: typeof crop) => ({
    height: rect.height / Math.max(1, output.height),
    width: rect.width / Math.max(1, output.width),
    x: rect.x / Math.max(1, output.width),
    y: rect.y / Math.max(1, output.height),
  });
  const controls: RecordingSceneControls = {
    ...recordingSceneEdits({
      camera: sceneCamera,
      clips,
      commit,
      getPositionMs,
      output,
      paneOf,
      sourceDurationMs,
    }),
    autoZoom: () => {
      planRecordingAutoZoom(
        artifactId,
        visibleScreenArea(output, frames.primary),
      )
        .then((planned) => {
          commitClips(
            withAutoZooms(editRef.current.sceneClips ?? EMPTY_CLIPS, planned),
          );
        })
        .catch((cause: unknown) => {
          console.error("Could not make the auto zooms", cause);
        });
    },
    clearAutoZooms: () => {
      commitClips(withAutoZooms(editRef.current.sceneClips ?? EMPTY_CLIPS, []));
    },
  };
  usePublishRecordingScene(
    "recording",
    isAvailable
      ? {
          boxes: current?.boxes
            ? {
                camera: current.boxes.camera ?? null,
                cameraBehind: current.boxes.cameraBehind ?? false,
                screen: current.boxes.screen,
              }
            : null,
          cameraAspect,
          canvasAspect: output.width / Math.max(1, output.height),
          composition: {
            // Where the camera is drawn, which can differ from its stored box.
            camera: sceneCamera
              ? share(drawnCameraFrame(cameraOverlay, sceneCamera.aspect))
              : null,
            screen: share(crop),
          },
          framing: currentPane?.framing ?? null,
          framingPane: currentPane?.pane ?? "screen",
          hasAutoZooms: clips.some((clip) => clip.auto),
          hasCamera,
          hasSceneAtPlayhead: current !== null,
          isBaked,
          preset: current?.preset ?? null,
          // A pane the scene does not round keeps the recording's radius.
          radius:
            current && currentPane
              ? (current.radius?.[currentPane.pane] ??
                (currentPane.pane === "camera"
                  ? cameraOverlay.radiusPercent
                  : output.radiusPercent))
              : null,
          screenAspect,
          variant: current ? (current.variant ?? {}) : null,
        }
      : null,
    controls,
  );
  return {
    canPlaceCamera: hasCamera && isBaked,
    clips,
    controls,
    current,
    isAvailable,
    onClipsChange: commit,
    onDraftChange: showDraft,
    selection,
  };
}
