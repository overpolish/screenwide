// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  croppedCustomPane,
  customCropWindow,
  pictureRect,
} from "./recording-scene-custom-crop";

const canvas = { height: 900, width: 1600 };
/** A 2:1 screen in a box at a tenth of the canvas, zoomed twice past covering
 * it: the clip `scene_reframe.rs` crops in place. */
const pane = {
  aspect: 2,
  box: { height: 450, width: 800, x: 160, y: 90 },
  framing: { focusX: 0.5, focusY: 0.5, zoom: 2 },
};

const expectRect = (
  actual: { height: number; width: number; x: number; y: number },
  expected: { height: number; width: number; x: number; y: number },
) => {
  expect(actual.x).toBeCloseTo(expected.x);
  expect(actual.y).toBeCloseTo(expected.y);
  expect(actual.width).toBeCloseTo(expected.width);
  expect(actual.height).toBeCloseTo(expected.height);
};

describe("pictureRect", () => {
  it("places the picture where the native preview draws it", () => {
    expectRect(pictureRect(pane), {
      height: 900,
      width: 1800,
      x: -340,
      y: -135,
    });
  });
});

describe("croppedCustomPane", () => {
  it("leaves the picture where it sits as the box changes shape", () => {
    const next = customCropWindow(pane.box, {
      anchor: { x: 0, y: 0 },
      delta: { x: -300, y: 0 },
      edges: 2,
      operation: "cropResize",
    });
    const cropped = croppedCustomPane(pane, { canvas, next, slides: false });
    expect(cropped.box.width).toBeCloseTo(500 / 1600);
    expect(cropped.box.height).toBeCloseTo(0.5);
    const held = {
      height: cropped.box.height * canvas.height,
      width: cropped.box.width * canvas.width,
      x: cropped.box.x * canvas.width,
      y: cropped.box.y * canvas.height,
    };
    expectRect(pictureRect({ ...pane, box: held, framing: cropped.framing }), {
      height: 900,
      width: 1800,
      x: -340,
      y: -135,
    });
  });

  it("holds a window dragged past the picture to the picture", () => {
    const next = customCropWindow(pane.box, {
      anchor: { x: 0, y: 0 },
      delta: { x: 0, y: -400 },
      edges: 0,
      operation: "cropMove",
    });
    const cropped = croppedCustomPane(pane, { canvas, next, slides: true });
    expect(cropped.box.y * canvas.height).toBeCloseTo(-135);
    expect(cropped.box.height * canvas.height).toBeCloseTo(450);
  });
});
