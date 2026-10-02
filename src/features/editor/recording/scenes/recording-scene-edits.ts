// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";

import { RecordingSceneControls } from "./recording-scene-channel";
import { customRecordingSceneClip } from "./recording-scene-custom";
import { framingWithin } from "./recording-scene-framing";
import { ScenePaneFraming } from "./recording-scene-pane-framing";
import { SceneCamera } from "./recording-scene-placement";
import {
  clipFromTemplate,
  sceneTemplateLayout,
} from "./recording-scene-template";
import {
  insertRecordingSceneClip,
  RecordingSceneClip,
  recordingSceneClipAt,
  RecordingScenePreset,
} from "./recording-scenes";

/**
 * What the Scene panel's asks do to `clips`: each changes the scene under
 * the playhead, read when it is asked, or adds one there, and hands the
 * clips that result to `commit`.
 */
export function recordingSceneEdits({
  camera,
  clips,
  commit,
  getPositionMs,
  output,
  paneOf,
  sourceDurationMs,
}: {
  /** The camera a scene can place: one drawn into the picture. */
  camera: SceneCamera | null;
  clips: RecordingSceneClip[];
  commit: (clips: RecordingSceneClip[]) => void;
  getPositionMs: () => number;
  output: ScreenshotOutputSettings;
  /** The pane of a clip the panel's zoom, position and radius reach. */
  paneOf: (clip: RecordingSceneClip) => ScenePaneFraming | null;
  sourceDurationMs: number;
}): RecordingSceneControls {
  const reframeCurrent = (
    reframe: (clip: RecordingSceneClip) => RecordingSceneClip,
  ) => {
    const target = recordingSceneClipAt(clips, getPositionMs());
    if (target)
      commit(
        clips.map((clip) => (clip.id === target.id ? reframe(clip) : clip)),
      );
  };
  const inserted = (preset: RecordingScenePreset, atMs: number) =>
    insertRecordingSceneClip({
      atMs,
      clips,
      id: crypto.randomUUID(),
      preset,
      sourceDurationMs,
    })?.clips;
  const canvasAspect = output.width / Math.max(1, output.height);
  // Changes the scene under the playhead, or where there is none adds a full
  // one there and changes that at once.
  const remakeAt = (
    remake: (clip: RecordingSceneClip) => RecordingSceneClip,
  ) => {
    const atMs = getPositionMs();
    const next = recordingSceneClipAt(clips, atMs)
      ? clips
      : inserted("full", atMs);
    const target = next ? recordingSceneClipAt(next, atMs) : null;
    if (!next || !target) return;
    commit(next.map((clip) => (clip.id === target.id ? remake(clip) : clip)));
  };
  return {
    // A preset chosen for a custom scene puts its panes back in the preset's
    // boxes. The variant stays, so a swapped pair stays swapped.
    choosePreset: (preset) => {
      const atMs = getPositionMs();
      if (recordingSceneClipAt(clips, atMs)) {
        reframeCurrent(({ boxes: _boxes, ...clip }) => ({ ...clip, preset }));
        return;
      }
      const next = inserted(preset, atMs);
      if (next) commit(next);
    },
    chooseTemplate: (template) => {
      remakeAt((clip) => clipFromTemplate(clip, { canvasAspect, template }));
    },
    customize: () => {
      remakeAt((clip) => customRecordingSceneClip({ camera, clip, output }));
    },
    remove: () => {
      const target = recordingSceneClipAt(clips, getPositionMs());
      if (target) commit(clips.filter((clip) => clip.id !== target.id));
    },
    // A custom scene goes back to its preset's boxes as well, and every pane
    // to the recording's radius. The variant is the layout, not a framing,
    // so it stays.
    resetFraming: () => {
      reframeCurrent(
        ({
          boxes: _boxes,
          camera: _camera,
          radius: _radius,
          screen: _screen,
          ...clip
        }) => clip,
      );
    },
    // The framing is held where the picture still fills its box, so a
    // position typed past the edge lands on the edge.
    setFraming: (framing) => {
      reframeCurrent((clip) => {
        const pane = paneOf(clip);
        if (!pane) return clip;
        return {
          ...clip,
          [pane.pane]: framingWithin(
            { ...pane.framing, ...framing },
            pane.box,
            pane.aspect,
          ),
        };
      });
    },
    setRadius: (radius) => {
      reframeCurrent((clip) => {
        const pane = paneOf(clip);
        if (!pane) return clip;
        return {
          ...clip,
          radius: {
            ...clip.radius,
            [pane.pane]: Math.min(50, Math.max(0, radius)),
          },
        };
      });
    },
    setVariant: (variant) => {
      reframeCurrent((clip) => ({
        ...clip,
        variant: { ...clip.variant, ...variant },
      }));
    },
    templateLayout: () => {
      const current = recordingSceneClipAt(clips, getPositionMs());
      return current ? sceneTemplateLayout(current, canvasAspect) : null;
    },
  };
}
