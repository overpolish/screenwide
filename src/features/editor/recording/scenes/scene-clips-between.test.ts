// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { canvasShare, percent } from "../../layer-measure";

import { RecordingSceneClip } from "./recording-scenes";
import { sceneClipsBetween } from "./scene-clips-between";

const scene: RecordingSceneClip = {
  endMs: 2000,
  id: "a",
  preset: "full",
  startMs: 0,
};

/** Scenes between two lists, the recording's own corners 1% of the canvas
 * for the screen and a circle for the camera, its pane 200 pixels across
 * on a canvas 1000 high. */
const clipsBetween = sceneClipsBetween({
  camera: (clip) => clip.radius?.camera ?? percent(50),
  canvas: 1_000,
  frames: () => ({ camera: 200, screen: 600 }),
  screen: (clip) => clip.radius?.screen ?? canvasShare(1),
});

describe("a camera's circle", () => {
  it("eases to a look as the looks ease, through the radius it draws", () => {
    // The circle draws 100 pixels, a tenth of the canvas; Round is 3%.
    const to = { ...scene, radius: { camera: canvasShare(3) } };
    const [half] = clipsBetween([scene], [to], 0.5);
    const camera = half.radius?.camera;
    expect(camera && "canvas" in camera ? camera.canvas : null).toBeCloseTo(
      6.5,
    );
  });
});

describe("sceneClipsBetween", () => {
  it("pads, rounds and turns a scene part of the way", () => {
    const to = {
      ...scene,
      inset: canvasShare(3),
      radius: { screen: canvasShare(3) },
      rotation: { x: 0, y: 20, z: 0 },
    };
    const [half] = clipsBetween([scene], [to], 0.5);
    expect(half.inset).toEqual(canvasShare(1.5));
    // From the recording's own corners, which the scene took until now.
    expect(half.radius).toEqual({ screen: canvasShare(2) });
    expect(half.rotation).toEqual({ x: 0, y: 10, z: 0 });
  });

  it("starts on the old scene and ends on the new one", () => {
    const to = { ...scene, inset: canvasShare(6) };
    expect(clipsBetween([scene], [to], 0)[0].inset).toEqual(canvasShare(0));
    expect(clipsBetween([scene], [to], 1)[0].inset).toEqual(canvasShare(6));
  });

  it("draws a scene only the new clips have as it is", () => {
    const added = { ...scene, id: "b", inset: canvasShare(6) };
    expect(clipsBetween([scene], [added], 0.5)).toEqual([added]);
  });

  it("leaves what it does not move as the new scene has it", () => {
    const [half] = clipsBetween([scene], [scene], 0.5);
    expect(half).toEqual(scene);
  });
});
