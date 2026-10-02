// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultScreenshotOutput } from "../../screenshot/screenshot-output";

import { arrangedRecordingScene } from "./recording-scene-arrangement";
import {
  customRecordingSceneClip,
  movedSceneBox,
} from "./recording-scene-custom";
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
    sourceMs: 5_000,
  });

const clip = (preset: RecordingSceneClip["preset"]): RecordingSceneClip => ({
  endMs: 10_000,
  id: "c",
  preset,
  startMs: 0,
});

describe("customRecordingSceneClip", () => {
  it.each(["full", "split-two-thirds", "picture-in-picture"] as const)(
    "keeps a %s scene's panes where they were",
    (preset) => {
      const before = arranged({
        ...clip(preset),
        screen: { focusX: 0.4, focusY: 0.6, zoom: 2 },
      });
      const custom = customRecordingSceneClip({
        camera,
        clip: before?.clip ?? clip(preset),
        output,
      });
      expect(custom.boxes?.camera).toBeDefined();
      const after = arranged(custom);
      expect(after?.output.cropX).toBeCloseTo(before?.output.cropX ?? NaN);
      expect(after?.output.cropWidth).toBeCloseTo(
        before?.output.cropWidth ?? NaN,
      );
      expect(after?.output.imageX).toBeCloseTo(before?.output.imageX ?? NaN);
      expect(after?.output.imageWidth).toBeCloseTo(
        before?.output.imageWidth ?? NaN,
      );
      expect(after?.cameraOverlay?.frameX).toBeCloseTo(
        before?.cameraOverlay?.frameX ?? NaN,
      );
      expect(after?.cameraOverlay?.cameraX).toBeCloseTo(
        before?.cameraOverlay?.cameraX ?? NaN,
      );
      expect(after?.cameraOverlay?.cameraWidth).toBeCloseTo(
        before?.cameraOverlay?.cameraWidth ?? NaN,
      );
    },
  );

  it("gives a full scene no camera box where the camera is not drawn in", () => {
    const custom = customRecordingSceneClip({
      camera: null,
      clip: clip("full"),
      output,
    });
    expect(custom.boxes).toEqual({
      screen: { height: 1, width: 1, x: 0, y: 0 },
    });
  });
});

describe("arrangedRecordingScene in a custom scene", () => {
  it("puts the panes in its boxes, as `custom_tests.rs` pins", () => {
    const scene = arranged({
      ...clip("split-two-thirds"),
      boxes: {
        camera: { height: 0.3, width: 0.2, x: 0.7, y: 0.6 },
        screen: { height: 0.5, width: 0.5, x: 0.1, y: 0.1 },
      },
    });
    expect(scene?.output).toMatchObject({
      cropHeight: 450,
      cropWidth: 800,
      cropX: 160,
      cropY: 90,
    });
    expect(scene?.cameraOverlay?.frameX).toBeCloseTo(1120);
    expect(scene?.cameraOverlay?.frameY).toBeCloseTo(540);
    expect(scene?.cameraOverlay?.frameWidth).toBeCloseTo(320);
    expect(scene?.cameraOverlay?.frameHeight).toBeCloseTo(270);
  });
});

describe("movedSceneBox", () => {
  const box = { height: 0.3, width: 0.2, x: 0.7, y: 0.6 };

  it("keeps a box dragged away with its middle on the canvas", () => {
    const moved = movedSceneBox(box, { delta: { x: 2, y: -2 }, scale: 1 });
    expect(moved.x).toBeCloseTo(0.9);
    expect(moved.y).toBeCloseTo(-0.15);
  });

  it("keeps a shrunk box's shape at the smallest size", () => {
    const shrunk = movedSceneBox(box, { delta: { x: 0, y: 0 }, scale: 0 });
    expect(shrunk.width).toBeCloseTo(0.02);
    expect(shrunk.height / shrunk.width).toBeCloseTo(1.5);
  });
});
