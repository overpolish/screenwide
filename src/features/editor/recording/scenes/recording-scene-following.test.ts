// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultScreenshotOutput } from "../../screenshot/screenshot-output";
import { faceAt } from "../preview/face-path";

import { recordingSceneEdits } from "./recording-scene-edits";
import { selectedPaneFraming } from "./recording-scene-pane-framing";
import {
  basePlacement,
  ownCameraPicture,
  Picture,
  SceneCamera,
  targeting,
} from "./recording-scene-placement";
import { RecordingSceneClip } from "./recording-scenes";

const output = defaultScreenshotOutput(1600, 900);
const face = { x: 0.62, y: 0.4 };
const overlay = {
  cameraWidth: 640,
  cameraX: 1200,
  cameraY: 600,
  followFace: true as const,
  frameHeight: 240,
  frameWidth: 240,
  frameX: 1080,
  frameY: 480,
  radius: { canvas: 0 },
};
const camera: SceneCamera = { aspect: 16 / 9, face, overlay };
// A custom scene's camera box, a fifth of the canvas, following the face
// with it a tenth of the picture right of the box's middle.
const scene: RecordingSceneClip = {
  boxes: {
    camera: { height: 0.3, width: 0.2, x: 0.7, y: 0.1 },
    screen: { height: 0.8, width: 0.6, x: 0.05, y: 0.1 },
  },
  camera: { focusX: 0.6, focusY: 0.5, zoom: 2 },
  endMs: 10_000,
  followFace: true,
  id: "s",
  preset: "split-two-thirds",
  startMs: 0,
};

const canvas = { height: output.height, width: output.width };
const placedCamera = (clip: RecordingSceneClip) =>
  targeting({ base: basePlacement(output, camera), camera, canvas })(clip)
    ?.camera;

/** Where the face lands on the canvas in `picture`, of the camera's
 * aspect. */
const faceOn = (picture: Picture) => ({
  x: picture.x + (face.x - 0.5) * picture.width,
  y: picture.y + ((face.y - 0.5) * picture.width) / camera.aspect,
});

describe("a camera following the face", () => {
  it("is held with the face in the middle of its own crop outside every scene", () => {
    const picture = ownCameraPicture(camera);
    const landed = faceOn(picture);
    expect(landed.x).toBeCloseTo(overlay.frameX + overlay.frameWidth / 2);
    expect(landed.y).toBeCloseTo(overlay.frameY + overlay.frameHeight / 2);
  });

  it("is placed in a scene with the face where its framing says", () => {
    const picture = placedCamera(scene);
    if (!picture) throw new Error("the scene places no camera");
    const box = { width: 0.2 * canvas.width, x: 0.7 * canvas.width };
    // The framing's focus is a tenth of the picture right of the middle, so
    // the face sits that far left of the box's middle.
    expect(faceOn(picture).x).toBeCloseTo(
      box.x + box.width / 2 - 0.1 * picture.width,
    );
  });

  it("keeps its picture where it is when the scene stops following", () => {
    const committed: RecordingSceneClip[][] = [];
    const edits = recordingSceneEdits({
      camera,
      chosenId: null,
      clips: [scene],
      commit: (next) => committed.push(next),
      faceNow: () => face,
      getTimelineMs: () => 5_000,
      lengthMs: 10_000,
      output,
      paneOf: (clip) =>
        selectedPaneFraming({ camera, clip, output, selectsCamera: true }),
    });
    edits.setFollowFace(false);
    expect(committed).toHaveLength(1);
    const [stopped] = committed[0];
    expect(stopped.followFace).toBe(false);
    const before = placedCamera(scene);
    const after = placedCamera(stopped);
    expect(after?.x).toBeCloseTo(before?.x ?? Number.NaN);
    expect(after?.y).toBeCloseTo(before?.y ?? Number.NaN);
    expect(after?.width).toBeCloseTo(before?.width ?? Number.NaN);
  });
});

describe("faceAt", () => {
  const path = {
    points: [
      [0.2, 0.3],
      [0.4, 0.5],
    ] as [number, number][],
    stepMs: 100,
  };

  it("reads between the samples and holds at either end", () => {
    expect(faceAt(path, 50)).toEqual({ x: 0.30000000000000004, y: 0.4 });
    expect(faceAt(path, -20)).toEqual({ x: 0.2, y: 0.3 });
    expect(faceAt(path, 500)).toEqual({ x: 0.4, y: 0.5 });
  });
});
