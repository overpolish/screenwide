// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  defaultScreenshotOutput,
  resizeScreenshotCanvas,
  resizeScreenshotWorkspaceCanvas,
  resizeScreenshotWorkspaceCanvasEdges,
  ScreenshotWorkspaceOutputSettings,
} from "./screenshot-output";

const placement = (settings: {
  cropHeight: number;
  cropWidth: number;
  cropX: number;
  cropY: number;
  imageWidth: number;
  imageX: number;
  imageY: number;
}) => ({
  cropHeight: settings.cropHeight,
  cropWidth: settings.cropWidth,
  cropX: settings.cropX,
  cropY: settings.cropY,
  imageWidth: settings.imageWidth,
  imageX: settings.imageX,
  imageY: settings.imageY,
});

const layer = (x: number, y: number) => ({
  ...defaultScreenshotOutput(1_600, 900),
  cropHeight: 300,
  cropWidth: 400,
  cropX: x,
  cropY: y,
  imageWidth: 400,
  imageX: x,
  imageY: y,
});

const workspace = (): ScreenshotWorkspaceOutputSettings => ({
  ...layer(100, 50),
  items: [
    { id: 0, output: layer(100, 50) },
    { id: 1, output: layer(900, 500) },
  ],
});

describe("resizing the canvas", () => {
  it("leaves a single output's layer exactly where it was", () => {
    const settings = layer(100, 50);
    const resized = resizeScreenshotCanvas({
      height: 2_000,
      settings,
      width: 3_000,
    });

    expect(resized.width).toBe(3_000);
    expect(resized.height).toBe(2_000);
    expect(placement(resized)).toStrictEqual(placement(settings));
  });

  it("leaves every workspace layer exactly where it was", () => {
    const settings = workspace();
    const resized = resizeScreenshotWorkspaceCanvas({
      height: 2_000,
      settings,
      width: 3_000,
    });

    expect(resized.width).toBe(3_000);
    expect(resized.height).toBe(2_000);
    for (const [index, item] of resized.items.entries()) {
      expect(item.output.width).toBe(3_000);
      expect(item.output.height).toBe(2_000);
      expect(placement(item.output)).toStrictEqual(
        placement(settings.items[index].output),
      );
    }
  });

  it("is the identity for layers when a frame drag grows then shrinks back", () => {
    const settings = workspace();
    // The far edges of both axes, dragged out by half the canvas and back.
    const grown = resizeScreenshotWorkspaceCanvasEdges({
      deltaX: 0.5,
      deltaY: 0.5,
      edges: 2 | 8,
      settings,
    });
    const shrunk = resizeScreenshotWorkspaceCanvasEdges({
      deltaX: -1 / 3,
      deltaY: -1 / 3,
      edges: 2 | 8,
      settings: grown,
    });

    expect(grown.width).toBe(2_400);
    expect(grown.height).toBe(1_350);
    expect(shrunk.width).toBe(settings.width);
    expect(shrunk.height).toBe(settings.height);
    for (const [index, item] of shrunk.items.entries())
      expect(placement(item.output)).toStrictEqual(
        placement(settings.items[index].output),
      );
  });

  it("shifts layers with the origin when a near edge is dragged", () => {
    const settings = workspace();
    const resized = resizeScreenshotWorkspaceCanvasEdges({
      deltaX: 0.25,
      deltaY: 0,
      edges: 1,
      settings,
    });

    // The left edge moved in by a quarter of the width, so the origin moved
    // right by that much and every layer is measured from the new corner:
    // on screen nothing moved.
    const shift = settings.width - 1_200;
    expect(resized.width).toBe(1_200);
    expect(resized.height).toBe(settings.height);
    for (const [index, item] of resized.items.entries()) {
      const before = placement(settings.items[index].output);
      expect(placement(item.output)).toStrictEqual({
        ...before,
        cropX: before.cropX - shift,
        imageX: before.imageX - shift,
      });
    }
  });

  it("holds every layer's screen place across a centred grow and shrink", () => {
    const settings = workspace();
    const grown = resizeScreenshotWorkspaceCanvasEdges({
      deltaX: 0.25,
      deltaY: 0,
      edges: 2 | (1 << 16),
      settings,
    });
    const back = resizeScreenshotWorkspaceCanvasEdges({
      deltaX: -0.25 * (settings.width / grown.width),
      deltaY: 0,
      edges: 2 | (1 << 16),
      settings: grown,
    });
    for (const [index, item] of back.items.entries())
      expect(placement(item.output)).toStrictEqual(
        placement(settings.items[index].output),
      );
  });
});
