// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { pictureRect } from "./recording-scene-custom-crop";
import {
  recenteredScenePane,
  sceneWindowSourceCrop,
} from "./recording-scene-recenter";

// A 1000 by 500 source drawn whole at 800 by 400 on a 1000 by 600 canvas,
// and a scene zooming twice into its middle in a 400 by 200 box.
const source = { height: 500, width: 1000 };
const canvas = { height: 600, width: 1000 };
const whole = { height: 400, width: 800, x: 100, y: 100 };
const layout = { crop: whole, image: whole, sourceCrop: whole };
const pane = {
  aspect: 2,
  box: { height: 200, width: 400, x: 100, y: 100 },
  framing: { focusX: 0.5, focusY: 0.5, zoom: 2 },
};

describe("sceneWindowSourceCrop", () => {
  it("is the part of the source the scene's box shows", () => {
    const window = sceneWindowSourceCrop(pane, layout);
    expect(window?.x).toBeCloseTo(0.25);
    expect(window?.y).toBeCloseTo(0.25);
    expect(window?.width).toBeCloseTo(0.5);
    expect(window?.height).toBeCloseTo(0.5);
  });

  it("keeps within the recording's own crop of the source", () => {
    const cropped = {
      ...layout,
      sourceCrop: { height: 400, width: 400, x: 500, y: 100 },
    };
    const window = sceneWindowSourceCrop(pane, cropped);
    expect(window?.x).toBeCloseTo(0.5);
    expect(window?.width).toBeCloseTo(0.25);
  });

  it("is null where the box shows none of the source", () => {
    const away = { ...layout, sourceCrop: { ...whole, width: 100 } };
    expect(sceneWindowSourceCrop(pane, away)).toBeNull();
  });
});

describe("recenteredScenePane", () => {
  // Content found at 300 to 500 across and 150 to 250 down the source,
  // up and left of the box's middle.
  const content = { height: 100, width: 200, x: 300, y: 150 };
  const centred = recenteredScenePane(pane, {
    canvas,
    content,
    layout,
    source,
  });

  it("crops the box to the content, in the middle of where it was", () => {
    expect(centred.box.width * canvas.width).toBeCloseTo(160);
    expect(centred.box.height * canvas.height).toBeCloseTo(80);
    expect((centred.box.x + centred.box.width / 2) * canvas.width).toBeCloseTo(
      300,
    );
    expect(
      (centred.box.y + centred.box.height / 2) * canvas.height,
    ).toBeCloseTo(200);
  });

  it("moves the picture with it, at the size it was drawn", () => {
    const box = {
      height: centred.box.height * canvas.height,
      width: centred.box.width * canvas.width,
      x: centred.box.x * canvas.width,
      y: centred.box.y * canvas.height,
    };
    const picture = pictureRect({ aspect: 2, box, framing: centred.framing });
    expect(picture.width).toBeCloseTo(800);
    // The content's middle, 40% across and down the source, is the box's.
    expect(picture.x + picture.width * 0.4).toBeCloseTo(300);
    expect(picture.y + picture.height * 0.4).toBeCloseTo(200);
  });
});
