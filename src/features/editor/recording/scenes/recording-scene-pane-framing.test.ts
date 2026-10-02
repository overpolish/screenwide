// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultScreenshotOutput } from "../../screenshot/screenshot-output";

import { recordingScenePanes } from "./recording-scene-geometry";
import { selectedPaneFraming } from "./recording-scene-pane-framing";
import { RecordingSceneClip } from "./recording-scenes";

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
    radiusPercent: 50,
  },
};
const split: RecordingSceneClip = {
  camera: { focusX: 0.6, focusY: 0.5, zoom: 1.5 },
  endMs: 10_000,
  id: "s",
  preset: "split-two-thirds",
  screen: { focusX: 0.5, focusY: 0.5, zoom: 2 },
  startMs: 0,
};

describe("selectedPaneFraming", () => {
  it("reaches the camera in the box the scene gives it while it is selected", () => {
    const pane = selectedPaneFraming({
      camera,
      clip: split,
      output,
      selectsCamera: true,
    });
    const { camera: box } = recordingScenePanes(
      "split-two-thirds",
      { height: 900, width: 1600 },
      {
        cameraAspect: 16 / 9,
        screenAspect: output.cropWidth / output.cropHeight,
      },
    );
    expect(pane?.pane).toBe("camera");
    expect(pane?.box).toEqual(box);
    expect(pane?.framing).toEqual(split.camera);
  });

  it("stays on the screen where no camera is drawn into the picture", () => {
    const pane = selectedPaneFraming({
      camera: null,
      clip: { ...split, preset: "full" },
      output,
      selectsCamera: true,
    });
    expect(pane?.pane).toBe("screen");
    expect(pane?.framing).toEqual(split.screen);
  });
});
