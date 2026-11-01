// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  croppedCustomPane,
  customCropWindow,
  pictureRect,
  shapedCustomPane,
  uncroppedCustomPane,
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
    const cropped = croppedCustomPane(pane, {
      canvas,
      moving: 2,
      next,
      slides: false,
    });
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
    const cropped = croppedCustomPane(pane, {
      canvas,
      moving: 0,
      next,
      slides: true,
    });
    expect(cropped.box.y * canvas.height).toBeCloseTo(-135);
    expect(cropped.box.height * canvas.height).toBeCloseTo(450);
  });

  it("stops a crop pulled smaller than the furthest zoom, the picture kept", () => {
    const next = customCropWindow(pane.box, {
      anchor: { x: 0, y: 0 },
      delta: { x: -790, y: -440 },
      edges: 2 | 8,
      operation: "cropResize",
    });
    const cropped = croppedCustomPane(pane, {
      canvas,
      moving: 2 | 8,
      next,
      slides: false,
    });
    // The picture is 1800 wide, so the smallest window spans 225 at zoom 8,
    // grown from the right edge pulled, the left edge staying put.
    expect(cropped.framing.zoom).toBeCloseTo(8);
    expect(cropped.box.x * canvas.width).toBeCloseTo(160);
    expect(cropped.box.width * canvas.width).toBeCloseTo(225);
    expectRect(
      pictureRect({
        ...pane,
        box: inPixels(cropped.box),
        framing: cropped.framing,
      }),
      { height: 900, width: 1800, x: -340, y: -135 },
    );
  });
});

const inPixels = (box: {
  height: number;
  width: number;
  x: number;
  y: number;
}) => ({
  height: box.height * canvas.height,
  width: box.width * canvas.width,
  x: box.x * canvas.width,
  y: box.y * canvas.height,
});

describe("shapedCustomPane", () => {
  it("cuts the box to the shape about its middle, the picture staying put", () => {
    const shaped = shapedCustomPane(pane, { aspect: "1:1", canvas });
    if (!shaped) throw new Error("expected a crop");
    expectRect(inPixels(shaped.box), {
      height: 450,
      width: 450,
      x: 335,
      y: 90,
    });
    expectRect(
      pictureRect({
        ...pane,
        box: inPixels(shaped.box),
        framing: shaped.framing,
      }),
      { height: 900, width: 1800, x: -340, y: -135 },
    );
  });

  it("leaves a free crop, or a box of that shape already, as it is", () => {
    expect(shapedCustomPane(pane, { aspect: "free", canvas })).toBeNull();
    const pictureShaped = {
      ...pane,
      box: { height: 400, width: 800, x: 160, y: 90 },
    };
    expect(
      shapedCustomPane(pictureShaped, { aspect: "original", canvas }),
    ).toBeNull();
  });
});

describe("uncroppedCustomPane", () => {
  it("grows the box to the whole picture's shape about its middle", () => {
    const square = {
      ...pane,
      box: { height: 300, width: 300, x: 400, y: 200 },
    };
    const whole = uncroppedCustomPane(square, { canvas });
    expectRect(inPixels(whole.box), {
      height: 300,
      width: 600,
      x: 250,
      y: 200,
    });
    expect(whole.framing.zoom).toBeCloseTo(1);
  });

  it("fits a box too big for the canvas and slides it onto it", () => {
    const tall = {
      ...pane,
      box: { height: 900, width: 200, x: 1500, y: 0 },
    };
    const whole = uncroppedCustomPane(tall, { canvas });
    expectRect(inPixels(whole.box), { height: 800, width: 1600, x: 0, y: 50 });
  });
});
