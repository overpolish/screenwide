// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type CropAspect } from "../../island/crop-aspect-choice";
import { UPRIGHT_ROTATION } from "../../layer-rotation";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import { timedSpanClipAt } from "../../timeline/tracks/timed-span-clips";

import { RecordingSceneControls } from "./recording-scene-channel";
import { customRecordingSceneClip } from "./recording-scene-custom";
import {
  shapedCustomPane,
  uncroppedCustomPane,
} from "./recording-scene-custom-crop";
import { framingWithin } from "./recording-scene-framing";
import { arrangedScenePane, swappedScenePanes } from "./recording-scene-order";
import {
  ScenePaneFraming,
  scenePaneFraming,
} from "./recording-scene-pane-framing";
import { scenePaneAtLayout } from "./recording-scene-pane-reset";
import { SceneCamera } from "./recording-scene-placement";
import { SceneChoice } from "./recording-scene-span";
import {
  clipFromTemplate,
  sceneTemplateLayout,
} from "./recording-scene-template";
import {
  insertRecordingSceneClip,
  MAX_SCENE_INSET,
  RECORDING_SCENE_MINIMUM_MS,
  RecordingSceneClip,
  rotatedSceneClip,
  WHOLE_FRAMING,
} from "./recording-scenes";

/**
 * What the Scene panel's asks do to `clips`: each changes the scene under
 * the playhead, read when it is asked, or adds one there, and hands the
 * clips that result to `commit`. A choice for a stretch of words in the
 * Transcript panel acts on the scene it names instead. The clips and the
 * playhead are both on the edited video, `lengthMs` long.
 */
export function recordingSceneEdits({
  camera,
  clips,
  commit,
  getTimelineMs,
  lengthMs,
  output,
  paneOf,
}: {
  /** The camera a scene can place: one drawn into the picture. */
  camera: SceneCamera | null;
  clips: RecordingSceneClip[];
  commit: (clips: RecordingSceneClip[]) => void;
  getTimelineMs: () => number;
  lengthMs: number;
  output: ScreenshotOutputSettings;
  /** The pane of a clip the panel's zoom, position and radius reach: the
   * one named, else the selected one. */
  paneOf: (
    clip: RecordingSceneClip,
    pane?: ScenePaneFraming["pane"],
  ) => ScenePaneFraming | null;
}): Omit<
  RecordingSceneControls,
  "autoZoom" | "clearAutoZooms" | "recenterScreen"
> {
  const reframeCurrent = (
    reframe: (clip: RecordingSceneClip) => RecordingSceneClip,
  ) => {
    const target = timedSpanClipAt(clips, getTimelineMs());
    if (target)
      commit(
        clips.map((clip) => (clip.id === target.id ? reframe(clip) : clip)),
      );
  };
  const canvasAspect = output.width / Math.max(1, output.height);
  const canvas = { height: output.height, width: output.width };
  // A preset chosen for a custom scene puts its panes back in the preset's
  // boxes. The variant stays, so a swapped pair stays swapped. Custom keeps
  // the panes where the scene puts them now.
  const arranged = (
    clip: RecordingSceneClip,
    choice: SceneChoice,
  ): RecordingSceneClip => {
    switch (choice.type) {
      case "custom":
        return customRecordingSceneClip({ camera, clip, output });
      case "preset": {
        const { boxes: _boxes, ...rest } = clip;
        return { ...rest, preset: choice.preset };
      }
      case "template":
        return clipFromTemplate(clip, {
          canvasAspect,
          template: choice.template,
        });
    }
  };
  // Changes the scene under the playhead, or where there is none adds a full
  // one there and changes that at once.
  const chooseAtPlayhead = (choice: SceneChoice) => {
    const atMs = getTimelineMs();
    const next = timedSpanClipAt(clips, atMs)
      ? clips
      : insertRecordingSceneClip({
          atMs,
          clips,
          id: crypto.randomUUID(),
          lengthMs,
          preset: "full",
        })?.clips;
    const target = next ? timedSpanClipAt(next, atMs) : null;
    if (!next || !target) return;
    commit(
      next.map((clip) =>
        clip.id === target.id ? arranged(clip, choice) : clip,
      ),
    );
  };
  // The crop is each scene's own: its screen's box and framing, cropped in
  // place as a custom scene's is, a preset's made custom first, where the
  // change alone moves nothing.
  const cropScreen = (aspect: CropAspect) => {
    reframeCurrent((clip) => {
      const pane = scenePaneFraming({ camera, clip, output, pane: "screen" });
      const shaped = pane ? shapedCustomPane(pane, { aspect, canvas }) : null;
      const owned = shaped
        ? customRecordingSceneClip({ camera, clip, output })
        : clip;
      return shaped && owned.boxes
        ? {
            ...owned,
            boxes: { ...owned.boxes, screen: shaped.box },
            screen: shaped.framing,
          }
        : clip;
    });
  };
  // A preset's box is the picture's shape already, so its crop goes with
  // the zoom; a custom scene's box grows back to the picture's shape.
  const resetScreenCrop = () => {
    reframeCurrent((clip) => {
      if (!clip.boxes) return { ...clip, screen: WHOLE_FRAMING };
      const pane = scenePaneFraming({ camera, clip, output, pane: "screen" });
      if (!pane) return clip;
      const whole = uncroppedCustomPane(pane, { canvas });
      return {
        ...clip,
        boxes: { ...clip.boxes, screen: whole.box },
        screen: whole.framing,
      };
    });
  };
  return {
    arrangePane: (pane, move) => {
      reframeCurrent((clip) => arrangedScenePane(clip, pane, move));
    },
    // The scene named is laid out anew and moved to the span while it is
    // still there, or a new one is added over it. Either lands only where the
    // span still fits between the other scenes, in whole milliseconds, and
    // is at least the shortest scene, unless a short scene keeps its place.
    chooseForSpan: (target, choice) => {
      const { endMs, startMs } = target;
      const changed =
        target.type === "change"
          ? clips.find((clip) => clip.id === target.id)
          : undefined;
      if (target.type === "change" && !changed) return;
      const keepsTiming =
        changed?.startMs === startMs && changed.endMs === endMs;
      if (
        !Number.isInteger(startMs) ||
        !Number.isInteger(endMs) ||
        startMs < 0 ||
        endMs > lengthMs ||
        (endMs - startMs < RECORDING_SCENE_MINIMUM_MS && !keepsTiming) ||
        clips.some(
          (clip) =>
            clip !== changed && clip.startMs < endMs && startMs < clip.endMs,
        )
      )
        return;
      const next = changed
        ? clips.map((clip) =>
            clip === changed
              ? arranged({ ...clip, endMs, startMs }, choice)
              : clip,
          )
        : [
            ...clips,
            arranged(
              { endMs, id: crypto.randomUUID(), preset: "full", startMs },
              choice,
            ),
          ];
      commit(next.sort((a, b) => a.startMs - b.startMs));
    },
    choosePreset: (preset) => {
      chooseAtPlayhead({ preset, type: "preset" });
    },
    chooseTemplate: (template) => {
      chooseAtPlayhead({ template, type: "template" });
    },
    cropScreen,
    remove: () => {
      const target = timedSpanClipAt(clips, getTimelineMs());
      if (target) commit(clips.filter((clip) => clip.id !== target.id));
    },
    resetPanePlacement: (pane) => {
      reframeCurrent((clip) =>
        scenePaneAtLayout(clip, pane, { camera, output }),
      );
    },
    resetScreenCrop,
    // Following starts with the face in the middle of the camera's box at
    // the zoom the camera has now; a drag or the Position fields then set
    // how far off the middle it sits. Stopping keeps that framing as it is.
    // Either way the scene says so itself, whatever the camera does outside
    // it.
    setFollowFace: (follow) => {
      reframeCurrent((clip) => {
        if (!follow) return { ...clip, followFace: false };
        const pane = paneOf(clip);
        const zoom =
          pane?.pane === "camera"
            ? pane.framing.zoom
            : (clip.camera?.zoom ?? 1);
        return {
          ...clip,
          camera: { focusX: 0.5, focusY: 0.5, zoom },
          followFace: true,
        };
      });
    },
    // The framing is held where the picture still fills its box, so a
    // position typed past the edge lands on the edge.
    setFraming: (framing, paneName) => {
      reframeCurrent((clip) => {
        const pane = paneOf(clip, paneName);
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
    // A scene pads its screen's box whatever lays it out, so a preset's
    // stays a preset's.
    setInset: (inset) => {
      reframeCurrent((clip) => {
        const { inset: _inset, ...rest } = clip;
        const share = Math.min(MAX_SCENE_INSET, Math.max(0, inset));
        return share > 0 ? { ...rest, inset: share } : rest;
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
    setRotation: (rotation) => {
      reframeCurrent((clip) =>
        rotatedSceneClip(clip, {
          ...UPRIGHT_ROTATION,
          ...clip.rotation,
          ...rotation,
        }),
      );
    },
    setVariant: (variant) => {
      reframeCurrent((clip) => ({
        ...clip,
        variant: { ...clip.variant, ...variant },
      }));
    },
    swapPanes: () => {
      reframeCurrent(swappedScenePanes);
    },
    templateLayout: () => {
      const current = timedSpanClipAt(clips, getTimelineMs());
      return current ? sceneTemplateLayout(current, canvasAspect) : null;
    },
  };
}
