// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultScreenshotOutput } from "../../screenshot/screenshot-output";

import { arrangedRecordingScene } from "./recording-scene-arrangement";
import { SceneRect, recordingScenePanes } from "./recording-scene-geometry";
import { RecordingScenePreset } from "./recording-scenes";

const canvas = { height: 900, width: 1600 };
const output = defaultScreenshotOutput(1600, 900);
/** A 180 point camera in the corner: the fixture `preset_tests.rs` uses. */
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

const arranged = (preset: RecordingScenePreset, sourceMs: number) =>
  arrangedRecordingScene({
    camera,
    clips: [{ endMs: 10_000, id: "p", preset, startMs: 1000 }],
    output,
    ranges: uncut,
    sourceMs,
  });

const shapes = { cameraAspect: 16 / 9, screenAspect: 2 };

/** Rounded to the hundredth `preset_tests.rs` pins. */
const rounded = (rect: SceneRect | null) =>
  rect && {
    height: Math.round(rect.height * 100) / 100,
    width: Math.round(rect.width * 100) / 100,
    x: Math.round(rect.x * 100) / 100,
    y: Math.round(rect.y * 100) / 100,
  };

describe("recordingScenePanes", () => {
  it("stacks each pane at its own shape, the camera whole above when swapped", () => {
    const panes = recordingScenePanes("stacked", canvas, {
      ...shapes,
      variant: { cameraSize: "two-thirds", swap: true },
    });
    expect(rounded(panes.camera)).toEqual({
      height: 433.33,
      width: 770.37,
      x: 414.81,
      y: 100,
    });
    expect(rounded(panes.screen)).toEqual({
      height: 216.67,
      width: 433.33,
      x: 583.33,
      y: 583.33,
    });
  });

  it("gives the camera two thirds of a side by side on the left at its own shape", () => {
    const panes = recordingScenePanes("split-two-thirds", canvas, {
      ...shapes,
      variant: { cameraSize: "two-thirds", swap: true },
    });
    expect(rounded(panes.camera)).toEqual({
      height: 506.25,
      width: 900,
      x: 100,
      y: 196.88,
    });
    expect(rounded(panes.screen)).toEqual({
      height: 225,
      width: 450,
      x: 1050,
      y: 337.5,
    });
  });
});

describe("arrangedRecordingScene with a pane hidden", () => {
  it("fills the canvas with the camera alone, as `preset_tests.rs` pins", () => {
    const scene = arranged("camera-only", 5000);
    expect(scene?.opacity).toEqual({ camera: 1, screen: 0 });
    expect(scene?.cameraOverlay).toMatchObject({
      frameHeight: 900,
      frameWidth: 1600,
      frameX: 0,
      frameY: 0,
      radiusPercent: 0,
    });
  });

  it("shrinks a camera it hides where it was as it fades", () => {
    const scene = arranged("screen-only", 1300);
    expect(scene?.opacity.camera).toBeCloseTo(0.5);
    const frame = scene?.cameraOverlay;
    expect(frame?.frameWidth).toBeCloseTo(180 * 0.625);
    expect((frame?.frameX ?? NaN) + (frame?.frameWidth ?? NaN) / 2).toBeCloseTo(
      1400,
    );
    expect(
      (frame?.frameY ?? NaN) + (frame?.frameHeight ?? NaN) / 2,
    ).toBeCloseTo(750);
  });
});
