// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  clipFromTemplate,
  fittedTemplateBoxes,
  isTemplateLayout,
  nextSceneTemplateName,
  SceneTemplate,
  sceneTemplateLayout,
} from "./recording-scene-template";
import { RecordingSceneClip } from "./recording-scenes";

const WIDE = 16 / 9;

const SCREEN = { height: 0.6, width: 0.6, x: 0.05, y: 0.2 };
const CAMERA = { height: 0.4, width: 0.225, x: 0.7, y: 0.3 };

/** A custom scene on a 16:9 canvas: the screen zoomed into its corner, the
 * camera beside it rounded to a circle. */
const custom: RecordingSceneClip = {
  boxes: { camera: CAMERA, screen: SCREEN },
  camera: { focusX: 0.5, focusY: 0.5, zoom: 1 },
  endMs: 6000,
  id: "c",
  preset: "full",
  radius: { camera: 50 },
  screen: { focusX: 0.2, focusY: 0.3, zoom: 2 },
  startMs: 1000,
};

const saved = (): SceneTemplate => {
  const layout = sceneTemplateLayout(custom, WIDE);
  if (!layout) throw new Error("a custom scene is kept as a layout");
  return { ...layout, id: "t", name: "Template 1" };
};

const plain: RecordingSceneClip = {
  endMs: 9000,
  id: "p",
  preset: "split-two-thirds",
  startMs: 7000,
};

describe("scene templates", () => {
  it("keeps the layout of a custom scene, not its zoom", () => {
    expect(saved()).toEqual({
      boxes: custom.boxes,
      canvasAspect: WIDE,
      id: "t",
      name: "Template 1",
      radius: { camera: 50 },
    });
    expect(sceneTemplateLayout(plain, WIDE)).toBeNull();
  });

  it("lays a scene out as it was saved, keeping the scene's own zoom", () => {
    const zoomed: RecordingSceneClip = {
      ...plain,
      screen: { focusX: 0.7, focusY: 0.3, zoom: 1.6 },
    };
    const laid = clipFromTemplate(zoomed, {
      canvasAspect: WIDE,
      template: saved(),
    });
    expect(laid.boxes?.screen).toEqual(custom.boxes?.screen);
    expect(laid.boxes?.camera?.x).toBeCloseTo(0.7);
    expect(laid.screen).toEqual(zoomed.screen);
    expect(laid.camera).toBeUndefined();
    expect(laid.radius).toEqual({ camera: 50 });
    expect(laid.preset).toBe("split-two-thirds");
    expect(
      isTemplateLayout(laid.boxes ?? { screen: CAMERA }, {
        canvasAspect: WIDE,
        template: saved(),
      }),
    ).toBe(true);
  });

  it("fits a wide layout into a tall canvas whole, each box its own shape", () => {
    const tall = 9 / 16;
    const boxes = fittedTemplateBoxes(saved(), tall);
    // In pixels of a 900 by 1600 canvas the screen keeps the shape it had in
    // a 1600 by 900 one, and sits where the layout's canvas is centred.
    const shape = (box: { height: number; width: number }, aspect: number) =>
      (box.width * aspect) / box.height;
    expect(shape(boxes.screen, tall)).toBeCloseTo(shape(SCREEN, WIDE));
    expect(shape(boxes.camera ?? SCREEN, tall)).toBeCloseTo(
      shape(CAMERA, WIDE),
    );
    expect(boxes.screen.x).toBeCloseTo(0.05);
    expect(boxes.screen.width).toBeCloseTo(0.6);
    // The layout's canvas is a 9:16 canvas's width wide and 81/256 of its
    // height tall, centred top to bottom.
    const top = (1 - tall / WIDE) / 2;
    expect(boxes.screen.y).toBeCloseTo(top + 0.2 * (tall / WIDE));
    expect(boxes.screen.height).toBeCloseTo(0.6 * (tall / WIDE));
  });

  it("names a new template one past the highest kept", () => {
    expect(nextSceneTemplateName([])).toBe("Template 1");
    expect(
      nextSceneTemplateName([
        { ...saved(), name: "Template 4" },
        { ...saved(), name: "Template 2" },
      ]),
    ).toBe("Template 5");
  });
});
