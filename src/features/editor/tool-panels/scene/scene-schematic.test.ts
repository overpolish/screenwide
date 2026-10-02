// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { sceneSchematic } from "./scene-schematic";

const CANVASES = [16 / 9, 4 / 3, 1, 9 / 16, 21 / 9];
/** The recording's own composition: the screen filling the canvas. */
const COMPOSITION = {
  camera: null,
  screen: { height: 1, width: 1, x: 0, y: 0 },
};

const arranged = (
  preset: "picture-in-picture" | "split-two-thirds",
  canvas: number,
  screenAspect: number,
) => {
  const { camera, screen } = sceneSchematic(preset, {
    cameraAspect: 16 / 9,
    canvasAspect: canvas,
    composition: COMPOSITION,
    screenAspect,
  });
  if (!camera || !screen)
    throw new Error("an arrangement always places both panes");
  return { camera, screen };
};

describe("sceneSchematic", () => {
  it.each(CANVASES)("keeps the screen's shape inside a %f frame", (canvas) => {
    for (const preset of ["picture-in-picture", "split-two-thirds"] as const) {
      const { camera, screen } = arranged(preset, canvas, 16 / 10);
      // Shares of width and height turn back into the screen's own shape.
      expect((screen.width * canvas) / screen.height).toBeCloseTo(16 / 10);
      for (const rect of [camera, screen]) {
        expect(rect.x).toBeGreaterThan(0);
        expect(rect.y).toBeGreaterThan(0);
        expect(rect.x + rect.width).toBeLessThan(1);
        expect(rect.y + rect.height).toBeLessThan(1);
      }
    }
  });

  it.each(CANVASES)(
    "leaves the same space either side of picture in picture in a %f frame",
    (canvas) => {
      const { camera, screen } = arranged("picture-in-picture", canvas, 16 / 9);
      expect(1 - (camera.y + camera.height)).toBeCloseTo(screen.y);
      expect(1 - (camera.x + camera.width)).toBeCloseTo(screen.x);
      // The camera overlaps the screen's corner rather than sitting beside it.
      expect(camera.x).toBeLessThan(screen.x + screen.width);
      expect(camera.y).toBeLessThan(screen.y + screen.height);
    },
  );
});
