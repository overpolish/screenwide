// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultScreenshotOutput } from "../../screenshot/screenshot-output";

import {
  croppedSceneScreen,
  uncroppedSceneScreen,
} from "./recording-scene-crop-edits";
import { recordingScenePanes } from "./recording-scene-geometry";
import { scenePaneFraming } from "./recording-scene-pane-framing";
import { RecordingSceneClip, WHOLE_FRAMING } from "./recording-scenes";

const output = defaultScreenshotOutput(1600, 900);
const camera = {
  aspect: 16 / 9,
  overlay: {
    cameraWidth: 320,
    cameraX: 1400,
    cameraY: 750,
    frameHeight: 180,
    frameWidth: 180,
    frameX: 1310,
    frameY: 660,
    radius: { percent: 50 },
  },
};
const split: RecordingSceneClip = {
  endMs: 10_000,
  id: "s",
  preset: "split-two-thirds",
  startMs: 0,
};

describe("croppedSceneScreen", () => {
  it("keeps a preset, laying its screen out at the crop's shape", () => {
    const cropped = croppedSceneScreen(split, "1:1", { camera, output });
    expect(cropped.preset).toBe("split-two-thirds");
    expect(cropped.boxes).toBeUndefined();
    expect(cropped.shapes?.screen).toBeCloseTo(1);
    const pane = scenePaneFraming({
      camera,
      clip: cropped,
      output,
      pane: "screen",
    });
    const { screen } = recordingScenePanes(
      "split-two-thirds",
      { height: 900, width: 1600 },
      { cameraAspect: 16 / 9, screenAspect: 1 },
    );
    // To the pixel: the crop's shape is taken from a box in output pixels.
    expect(pane?.box.width).toBeCloseTo(screen?.width ?? 0, 0);
    expect(pane?.box.height).toBeCloseTo(screen?.height ?? 0, 0);
    expect(pane?.box.x).toBeCloseTo(screen?.x ?? 0, 0);
  });

  it("gives a preset's screen back its picture's shape when uncropped", () => {
    const cropped = croppedSceneScreen(
      { ...split, shapes: { camera: 1 } },
      "1:1",
      { camera, output },
    );
    const uncropped = uncroppedSceneScreen(cropped, { camera, output });
    expect(uncropped.screen).toEqual(WHOLE_FRAMING);
    expect(uncropped.shapes).toEqual({ camera: 1 });
  });
});
