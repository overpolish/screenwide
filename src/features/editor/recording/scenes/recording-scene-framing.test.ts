// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  drawnCameraFrame,
  framingFromWindow,
  reframeWindow,
} from "./recording-scene-framing";

const ASPECT = 16 / 9;
/** A square camera box, whose 16:9 picture overhangs it either side. */
const BOX = { height: 180, width: 180, x: 100, y: 100 };

describe("reframeWindow", () => {
  it("lays the whole picture over the box from its middle", () => {
    // `scene_reframe.rs` draws the camera at the same place, so the window
    // sits over the picture the preview shows.
    const { image } = reframeWindow({
      aspect: ASPECT,
      box: BOX,
      framing: { focusX: 0.5, focusY: 0.5, zoom: 1 },
    });
    expect(image).toEqual({ height: 180, width: 320, x: 30, y: 100 });
  });

  it("shows a smaller window the further a scene zooms", () => {
    const { window } = reframeWindow({
      aspect: ASPECT,
      box: BOX,
      framing: { focusX: 0.5, focusY: 0.5, zoom: 2 },
    });
    expect(window).toEqual({ height: 90, width: 90, x: 145, y: 145 });
  });

  it("gives back the framing a window was laid out from", () => {
    const framing = { focusX: 0.3, focusY: 0.6, zoom: 2.5 };
    const { window } = reframeWindow({ aspect: ASPECT, box: BOX, framing });
    const back = framingFromWindow({ aspect: ASPECT, box: BOX, window });
    expect(back.focusX).toBeCloseTo(framing.focusX);
    expect(back.focusY).toBeCloseTo(framing.focusY);
    expect(back.zoom).toBeCloseTo(framing.zoom);
  });

  it("holds a window dragged past the picture's edge at the edge", () => {
    const framing = framingFromWindow({
      aspect: ASPECT,
      box: BOX,
      window: { height: 90, width: 90, x: -500, y: 145 },
    });
    const { image, window } = reframeWindow({
      aspect: ASPECT,
      box: BOX,
      framing,
    });
    expect(window.x).toBeCloseTo(image.x);
  });
});

describe("drawnCameraFrame", () => {
  it("holds a box stored past the camera inside it, as `framing/tests.rs` pins", () => {
    const frame = drawnCameraFrame(
      {
        cameraWidth: 480,
        cameraX: 400,
        cameraY: 300,
        frameHeight: 200,
        frameWidth: 300,
        frameX: 100,
        frameY: 120,
        radiusPercent: 0,
      },
      ASPECT,
    );
    expect(frame).toEqual({ height: 200, width: 300, x: 160, y: 165 });
  });
});
