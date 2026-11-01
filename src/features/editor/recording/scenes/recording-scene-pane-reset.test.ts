// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultScreenshotOutput } from "../../screenshot/screenshot-output";

import {
  customRecordingSceneClip,
  movedSceneBox,
} from "./recording-scene-custom";
import {
  isScenePaneAtLayout,
  scenePaneAtLayout,
} from "./recording-scene-pane-reset";
import { RecordingSceneClip, SceneBox } from "./recording-scenes";

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
const look = { camera, output };

const preset = (kind: RecordingSceneClip["preset"]): RecordingSceneClip => ({
  endMs: 10_000,
  id: "c",
  preset: kind,
  startMs: 0,
});

const expectBox = (actual: SceneBox | undefined, expected: SceneBox) => {
  expect(actual).toBeDefined();
  for (const key of ["height", "width", "x", "y"] as const)
    expect(actual?.[key], key).toBeCloseTo(expected[key], 6);
};

describe("scenePaneAtLayout", () => {
  it("leaves a preset scene's panes, which are in its boxes already", () => {
    const clip = preset("split-two-thirds");
    expect(isScenePaneAtLayout(clip, "screen", look)).toBe(true);
    expect(isScenePaneAtLayout(clip, "camera", look)).toBe(true);
    expect(scenePaneAtLayout(clip, "screen", look)).toBe(clip);
  });

  it.each(["screen", "camera"] as const)(
    "puts a moved and sized %s back in the box its preset gives it",
    (pane) => {
      const custom = customRecordingSceneClip({
        camera,
        clip: preset("split-two-thirds"),
        output,
      });
      const laidOut = custom.boxes?.[pane];
      expect(laidOut).toBeDefined();
      if (!custom.boxes || !laidOut) return;
      expect(isScenePaneAtLayout(custom, pane, look)).toBe(true);
      const moved: RecordingSceneClip = {
        ...custom,
        boxes: {
          ...custom.boxes,
          [pane]: movedSceneBox(laidOut, {
            delta: { x: 0.05, y: -0.03 },
            scale: 0.7,
          }),
        },
      };
      expect(isScenePaneAtLayout(moved, pane, look)).toBe(false);
      const reset = scenePaneAtLayout(moved, pane, look);
      expectBox(reset.boxes?.[pane], laidOut);
      expect(isScenePaneAtLayout(reset, pane, look)).toBe(true);
    },
  );

  it("keeps a pane cropped in place at its shape, fitted in its box", () => {
    const custom = customRecordingSceneClip({
      camera,
      clip: preset("split-two-thirds"),
      output,
    });
    const laidOut = custom.boxes?.screen;
    if (!custom.boxes || !laidOut) throw new Error("a custom split has boxes");
    // Half the box's width cropped away, then moved aside.
    const cropped: RecordingSceneClip = {
      ...custom,
      boxes: {
        ...custom.boxes,
        screen: { ...laidOut, width: laidOut.width / 2, x: 0.02 },
      },
      screen: { focusX: 0.25, focusY: 0.5, zoom: 2 },
    };
    const reset = scenePaneAtLayout(cropped, "screen", look);
    const box = reset.boxes?.screen;
    // Half as wide as the layout's box against its full height: fitted, it
    // fills the height and stands in the middle of the width.
    expectBox(box, {
      height: laidOut.height,
      width: laidOut.width / 2,
      x: laidOut.x + laidOut.width / 4,
      y: laidOut.y,
    });
    expect(reset.screen).toEqual(cropped.screen);
    expect(reset.boxes?.camera).toEqual(custom.boxes.camera);
  });

  it("puts a full scene's screen back where the recording places it", () => {
    const custom = customRecordingSceneClip({
      camera,
      clip: preset("full"),
      output,
    });
    if (!custom.boxes) throw new Error("a custom full scene has boxes");
    const moved: RecordingSceneClip = {
      ...custom,
      boxes: {
        ...custom.boxes,
        screen: { height: 0.4, width: 0.4, x: 0.1, y: 0.1 },
      },
    };
    const reset = scenePaneAtLayout(moved, "screen", look);
    // The recording's own screen fills this canvas, and the box, still the
    // screen's shape, fills it again.
    expectBox(reset.boxes?.screen, { height: 1, width: 1, x: 0, y: 0 });
  });
});
