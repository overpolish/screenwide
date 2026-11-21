// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { measureLike, points } from "../../layer-measure";
import { UPRIGHT_ROTATION } from "../../layer-rotation";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import { timedSpanClipAt } from "../../timeline/tracks/timed-span-clips";
import {
  applyBackgroundToOutput,
  backgroundFromOutput,
} from "../../tools/frame/background-adapters";

import { RecordingSceneControls } from "./recording-scene-channel";
import {
  croppedSceneScreen,
  uncroppedSceneScreen,
} from "./recording-scene-crop-edits";
import { customRecordingSceneClip } from "./recording-scene-custom";
import {
  type Face,
  followingFace,
  framingWithin,
} from "./recording-scene-framing";
import { arrangedScenePane, swappedScenePanes } from "./recording-scene-order";
import { ScenePaneFraming } from "./recording-scene-pane-framing";
import { scenePaneAtLayout } from "./recording-scene-pane-reset";
import { sceneWithPaneShown } from "./recording-scene-pane-shown";
import { SceneCamera } from "./recording-scene-placement";
import { clipsWithSpanChosen, SceneChoice } from "./recording-scene-span";
import {
  clipFromTemplate,
  sceneTemplateLayout,
} from "./recording-scene-template";
import {
  insertRecordingSceneClip,
  RecordingSceneClip,
  rotatedSceneClip,
} from "./recording-scenes";

/**
 * What the Scene panel's asks do to `clips`: each changes the scene in hand,
 * the one chosen on the strip or else the one under the playhead, read when
 * it is asked, or adds one at the playhead where there is none, and hands
 * the clips that result to `commit`. What is done on the picture (a pane's
 * order, a crop) changes the scene under the playhead, which the picture
 * shows. A choice for a stretch of words in the Transcript panel acts on the
 * scene it names instead. The clips and the playhead are both on the edited
 * video, `lengthMs` long.
 */
export function recordingSceneEdits({
  camera,
  chosenId,
  clips,
  commit,
  faceNow,
  getTimelineMs,
  lengthMs,
  output,
  paneOf,
}: {
  /** The camera a scene can place: one drawn into the picture. */
  camera: SceneCamera | null;
  /** The scene chosen on the strip, where one alone is. */
  chosenId: string | null;
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
  /** Where the face is over the camera at the playhead, once it is found,
   * which a scene that stops following keeps its picture by. */
  faceNow?: () => Face | null;
}): Omit<
  RecordingSceneControls,
  "autoZoom" | "clearAutoZooms" | "recenterScreen"
> {
  const atPlayhead = () => timedSpanClipAt(clips, getTimelineMs());
  const inHand = () =>
    clips.find((clip) => clip.id === chosenId) ?? atPlayhead();
  const reframeOf =
    (target: () => RecordingSceneClip | null) =>
    (reframe: (clip: RecordingSceneClip) => RecordingSceneClip) => {
      const found = target();
      if (found)
        commit(
          clips.map((clip) => (clip.id === found.id ? reframe(clip) : clip)),
        );
    };
  const reframeCurrent = reframeOf(inHand);
  const reframeShown = reframeOf(atPlayhead);
  const canvasAspect = output.width / Math.max(1, output.height);
  // A preset chosen for a custom scene puts its panes back in the preset's
  // boxes. The variant stays, so a swapped pair stays swapped. Custom keeps
  // the panes where the scene puts them now. A layout chosen is the one the
  // scene has from then on, so a pane turned off before no longer leads back
  // to another.
  const arranged = (
    scene: RecordingSceneClip,
    choice: SceneChoice,
  ): RecordingSceneClip => {
    const { layoutBefore: _layoutBefore, ...clip } = scene;
    switch (choice.type) {
      case "custom":
        return customRecordingSceneClip({ camera, clip, output });
      case "preset": {
        const { boxes: _boxes, ...rest } = clip;
        return { ...rest, preset: choice.preset };
      }
      case "template": {
        // A template's boxes carry their own shapes.
        const { shapes: _shapes, ...rest } = clip;
        return clipFromTemplate(rest, {
          canvasAspect,
          template: choice.template,
        });
      }
    }
  };
  // Changes the scene in hand, or where there is none adds a full one at
  // the playhead and changes that at once.
  const chooseInHand = (choice: SceneChoice) => {
    const atMs = getTimelineMs();
    const held = inHand();
    const next = held
      ? clips
      : insertRecordingSceneClip({
          atMs,
          clips,
          id: crypto.randomUUID(),
          lengthMs,
          preset: "full",
        })?.clips;
    const target = held ?? (next ? timedSpanClipAt(next, atMs) : null);
    if (!next || !target) return;
    commit(
      next.map((clip) =>
        clip.id === target.id ? arranged(clip, choice) : clip,
      ),
    );
  };
  return {
    arrangePane: (pane, move) => {
      reframeShown((clip) => arrangedScenePane(clip, pane, move));
    },
    chooseForSpan: (target, choice) => {
      const next = clipsWithSpanChosen({
        arrange: (clip) => arranged(clip, choice),
        clips,
        lengthMs,
        target,
      });
      if (next) commit(next);
    },
    choosePreset: (preset) => {
      chooseInHand({ preset, type: "preset" });
    },
    chooseTemplate: (template) => {
      chooseInHand({ template, type: "template" });
    },
    cropScreen: (aspect) => {
      reframeShown((clip) =>
        croppedSceneScreen(clip, aspect, { camera, output }),
      );
    },
    remove: () => {
      const target = inHand();
      if (target) commit(clips.filter((clip) => clip.id !== target.id));
    },
    resetPanePlacement: (pane) => {
      reframeCurrent((clip) =>
        scenePaneAtLayout(clip, pane, { camera, output }),
      );
    },
    resetScreenCrop: () => {
      reframeShown((clip) => uncroppedSceneScreen(clip, { camera, output }));
    },
    // Kept as the canvas would draw it, so a composition chosen without its
    // blobs takes the ones the canvas would give it. Unset, the scene shows
    // the recording's background again, a later change to it included.
    setBackground: (background) => {
      reframeCurrent((clip) => {
        const { background: _background, ...rest } = clip;
        return background === null
          ? rest
          : {
              ...rest,
              background: backgroundFromOutput(
                applyBackgroundToOutput(output, background),
              ),
            };
      });
    },
    // Unset, the scene shows or hides the cursor as the recording does.
    setCursor: (shown) => {
      reframeCurrent((clip) => {
        const { cursor: _cursor, ...rest } = clip;
        return shown === null ? rest : { ...rest, cursor: shown };
      });
    },
    // Following starts with the face in the middle of the camera's box at
    // the zoom the camera has now; a drag or the Position fields then set
    // how far off the middle it sits. Stopping keeps the picture where the
    // face had it, written down as the framing it then shows, since a
    // framing that says where the face sits says something else once the
    // face is not followed. Either way the scene says so itself, whatever
    // the camera does outside it.
    setFollowFace: (follow) => {
      reframeCurrent((clip) => {
        const pane = paneOf(clip);
        const camera = pane?.pane === "camera" ? pane : null;
        if (!follow) {
          const face = faceNow?.();
          return {
            ...clip,
            ...(camera && face
              ? {
                  camera: framingWithin(
                    followingFace(camera.framing, face),
                    camera.box,
                    camera.aspect,
                  ),
                }
              : {}),
            followFace: false,
          };
        }
        const zoom = camera?.framing.zoom ?? clip.camera?.zoom ?? 1;
        return {
          ...clip,
          camera: { focusX: 0.5, focusY: 0.5, zoom },
          followFace: true,
        };
      });
    },
    // A scene pads its screen's box whatever lays it out, so a preset's
    // stays a preset's.
    setInset: (inset) => {
      reframeCurrent((clip) => {
        const { inset: _inset, ...rest } = clip;
        return inset > 0 ? { ...rest, inset: points(inset) } : rest;
      });
    },
    setPaneShown: (pane, shown) => {
      reframeCurrent((clip) => sceneWithPaneShown(clip, { pane, shown }));
    },
    // The screen's corners are in the unit the recording's own are in.
    setRadius: (radius) => {
      reframeCurrent((clip) => {
        const pane = paneOf(clip);
        if (!pane) return clip;
        const share = Math.min(50, Math.max(0, radius));
        const screen = measureLike(output.radius, Math.max(0, radius));
        return {
          ...clip,
          radius: {
            ...clip.radius,
            ...(pane.pane === "screen" ? { screen } : { camera: share }),
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
      const current = inHand();
      return current ? sceneTemplateLayout(current, canvasAspect) : null;
    },
  };
}
