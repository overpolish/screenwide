// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultScreenshotOutput } from "../../screenshot/screenshot-output";

import { arrangedRecordingScene } from "./recording-scene-arrangement";
import { customRecordingSceneClip } from "./recording-scene-custom";
import { sceneWithPaneShown } from "./recording-scene-pane-shown";
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
const uncut = [{ playbackRate: 1, sourceEndMs: 20_000, sourceStartMs: 0 }];

const arranged = (clip: RecordingSceneClip) =>
  arrangedRecordingScene({
    camera,
    clips: [clip],
    output,
    ranges: uncut,
    screen: { height: output.height, scaleFactor: 1, width: output.width },
    sourceMs: 5_000,
  });

const clip = (preset: RecordingSceneClip["preset"]): RecordingSceneClip => ({
  endMs: 10_000,
  id: "c",
  preset,
  startMs: 0,
});

const shown = (
  scene: RecordingSceneClip,
  pane: "camera" | "screen",
  isShown: boolean,
) => sceneWithPaneShown(scene, { pane, shown: isShown });

describe("a scene's panes shown and hidden", () => {
  it("keeps a screen-only scene's camera hidden once its screen is changed", () => {
    const custom = customRecordingSceneClip({
      camera,
      clip: clip("screen-only"),
      output,
    });
    expect(arranged(custom)?.opacity).toEqual({ camera: 0, screen: 1 });
  });

  it("gives a layout without its camera the screen alone, and back", () => {
    const hidden = shown(clip("picture-in-picture"), "camera", false);
    expect(hidden).toEqual({
      ...clip("screen-only"),
      layoutBefore: "picture-in-picture",
    });
    expect(arranged(hidden)?.output.cropWidth).toBeCloseTo(
      arranged(clip("screen-only"))?.output.cropWidth ?? NaN,
    );
    expect(shown(hidden, "camera", true)).toEqual(clip("picture-in-picture"));
  });

  it("gives a layout without its screen the camera alone", () => {
    const scene = shown(clip("split-two-thirds"), "screen", false);
    expect(scene.preset).toBe("camera-only");
    expect(arranged(scene)?.opacity).toEqual({ camera: 1, screen: 0 });
  });

  it("shows a screen-only scene's camera where the recording puts it", () => {
    const scene = shown(clip("screen-only"), "camera", true);
    expect(scene).toEqual(clip("full"));
    expect(arranged(scene)?.cameraOverlay?.frameX).toBeCloseTo(
      camera.overlay.frameX,
    );
  });

  it("leads back to the fullest layout through both panes turned off", () => {
    const screenOnly = shown(clip("picture-in-picture"), "camera", false);
    const blank = shown(screenOnly, "screen", false);
    expect(blank.preset).toBe("blank");
    const back = shown(shown(blank, "screen", true), "camera", true);
    expect(back).toEqual(clip("picture-in-picture"));
  });

  it("keeps a custom scene's panes where they were put", () => {
    const custom = customRecordingSceneClip({
      camera,
      clip: clip("picture-in-picture"),
      output,
    });
    const hidden = shown(custom, "camera", false);
    expect(hidden.boxes?.screen).toEqual(custom.boxes?.screen);
    expect(arranged(hidden)?.opacity).toEqual({ camera: 0, screen: 1 });
    expect(shown(hidden, "camera", true)).toEqual(custom);
  });

  it("leaves a scene that already shows the pane as it is", () => {
    expect(shown(clip("picture-in-picture"), "camera", true)).toEqual(
      clip("picture-in-picture"),
    );
  });
});
