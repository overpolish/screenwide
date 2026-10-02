// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultScreenshotOutput } from "../../screenshot/screenshot-output";

import { arrangedRecordingScene } from "./recording-scene-arrangement";
import { reframed } from "./recording-scene-framing";
import { recordingScenePanes } from "./recording-scene-geometry";
import { RecordingSceneClip, SceneFraming } from "./recording-scenes";

const ASPECT = 16 / 9;
const output = { ...defaultScreenshotOutput(1600, 900), cropHeight: 800 };
/** A square window on the camera in the recording's own composition: the
 * fixture `framing/tests.rs` uses. */
const overlay = {
  cameraWidth: 480,
  cameraX: 1340,
  cameraY: 750,
  frameHeight: 180,
  frameWidth: 180,
  frameX: 1310,
  frameY: 660,
  radiusPercent: 50,
};
/** Zoomed to 1.5 times what covers its box, the face right of the middle. */
const FACE: SceneFraming = { focusX: 0.625, focusY: 0.5, zoom: 1.5 };
const split: RecordingSceneClip = {
  camera: FACE,
  endMs: 10_000,
  id: "a",
  preset: "split-two-thirds",
  startMs: 0,
};
const zoom: RecordingSceneClip = {
  endMs: 10_000,
  id: "z",
  preset: "full",
  screen: { focusX: 0.5, focusY: 0.5, zoom: 2 },
  startMs: 0,
};
const uncut = [{ playbackRate: 1, sourceEndMs: 20_000, sourceStartMs: 0 }];

const arranged = (
  sourceMs: number,
  {
    clip = split,
    hasCamera = true,
    ranges = uncut,
  }: {
    clip?: RecordingSceneClip;
    hasCamera?: boolean;
    ranges?: typeof uncut;
  } = {},
) =>
  arrangedRecordingScene({
    camera: hasCamera ? { aspect: ASPECT, overlay } : null,
    clips: [clip],
    output,
    ranges,
    sourceMs,
  });

describe("arrangedRecordingScene", () => {
  it("leaves the composition alone outside every clip", () => {
    expect(arranged(12_000)).toBeNull();
  });

  it("places the panes and frames the camera where the preview draws them", () => {
    const scene = arranged(5_000);
    const panes = recordingScenePanes(
      "split-two-thirds",
      { height: 900, width: 1600 },
      { cameraAspect: ASPECT, screenAspect: 2 },
    );
    expect(scene?.output.cropX).toBeCloseTo(panes.screen?.x ?? NaN);
    expect(scene?.output.cropWidth).toBeCloseTo(panes.screen?.width ?? NaN);
    expect(scene?.cameraOverlay?.frameX).toBeCloseTo(panes.camera?.x ?? NaN);
    // `framing/tests.rs` pins the same numbers, so the selection lands where
    // the preview draws.
    expect(scene?.cameraOverlay?.cameraX).toBeCloseTo(1190.625);
    expect(scene?.cameraOverlay?.cameraY).toBeCloseTo(450);
    expect(scene?.cameraOverlay?.cameraWidth).toBeCloseTo(675);
  });

  it("is halfway there halfway through its arrival", () => {
    const scene = arranged(300);
    const settled = arranged(5_000);
    expect(scene?.output.cropX).toBeCloseTo(
      (output.cropX + (settled?.output.cropX ?? 0)) / 2,
    );
  });

  it("times its arrival on screen time, not recording time", () => {
    const doubled = [
      { playbackRate: 2, sourceEndMs: 20_000, sourceStartMs: 0 },
    ];
    // 600 ms of a doubled stretch plays in 300 ms: halfway through arriving.
    expect(arranged(600, { ranges: doubled })?.output.cropX).toBeCloseTo(
      arranged(300)?.output.cropX ?? 0,
    );
  });

  it("zooms the screen inside the box the recording gives it", () => {
    const scene = arranged(5_000, { clip: zoom });
    expect(scene?.output.cropX).toBeCloseTo(output.cropX);
    expect(scene?.output.cropWidth).toBeCloseTo(output.cropWidth);
    expect(scene?.output.imageWidth).toBeCloseTo(output.imageWidth * 2);
    // The camera stays as the recording frames it.
    expect(scene?.cameraOverlay).toEqual(overlay);
  });

  it("moves the camera's picture with the pointer when it is dragged", () => {
    const before = arranged(5_000);
    const target = before?.cameraTarget;
    if (!target) throw new Error("the split has a camera to reframe");
    const after = arranged(5_000, {
      clip: {
        ...split,
        camera: reframed({
          aspect: ASPECT,
          delta: { x: -24, y: 10 },
          frame: target.frame,
          framing: target.framing,
          scale: 1,
        }),
      },
    });
    expect(
      (after?.cameraOverlay?.cameraX ?? 0) -
        (before.cameraOverlay?.cameraX ?? 0),
    ).toBeCloseTo(-24);
    expect(
      (after?.cameraOverlay?.cameraY ?? 0) -
        (before.cameraOverlay?.cameraY ?? 0),
    ).toBeCloseTo(10);
  });

  it("plays only full scenes without a camera", () => {
    expect(arranged(5_000, { hasCamera: false })).toBeNull();
    const scene = arranged(5_000, { clip: zoom, hasCamera: false });
    expect(scene?.output.imageWidth).toBeCloseTo(output.imageWidth * 2);
    expect(scene?.cameraOverlay).toBeNull();
  });
});

describe("a scene's corner radius", () => {
  const rounded = (sourceMs: number) =>
    arrangedRecordingScene({
      camera: { aspect: ASPECT, overlay },
      clips: [
        {
          endMs: 10_000,
          id: "r",
          preset: "split-two-thirds",
          radius: { screen: 30 },
          startMs: 1000,
        },
      ],
      output: { ...output, radiusPercent: 10 },
      ranges: uncut,
      sourceMs,
    });

  it("rounds its panes and leaves the rest to the recording, as `radius_tests.rs` pins", () => {
    const scene = rounded(5000);
    expect(scene?.output.radiusPercent).toBe(30);
    expect(scene?.cameraOverlay?.radiusPercent).toBe(50);
  });

  it("eases its radius in with its arrival", () => {
    expect(rounded(1300)?.output.radiusPercent).toBeCloseTo(20);
  });
});
