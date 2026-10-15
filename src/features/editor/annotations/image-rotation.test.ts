// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { imageShape } from "./annotation-shape-parsers";
import { imageRotation, withImageRotation } from "./image-rotation";

const image = imageShape({
  angle: 0,
  aspect: 1,
  asset: "image:0123456789abcdef0123456789abcdef",
  center: { x: 40, y: 30 },
  size: 96,
});

describe("an image's rotation", () => {
  if (!image) throw new Error("a readable image");

  it("turns counter-clockwise about Z as its angle runs clockwise", () => {
    const turned = withImageRotation(image, { x: 0, y: 0, z: 90 });
    expect(turned.angle).toBeCloseTo(-Math.PI / 2);
    expect(turned.tilt).toBeUndefined();
    expect(imageRotation(turned).z).toBeCloseTo(90);
  });

  it("keeps a tilt in depth, and drops it once both axes are back", () => {
    const tilted = withImageRotation(image, { x: 20, y: -30, z: 0 });
    expect(tilted.tilt).toEqual({ x: 20, y: -30 });
    expect(imageShape(tilted)?.tilt).toEqual({ x: 20, y: -30 });
    const flat = withImageRotation(tilted, { x: 0, y: 0, z: 0 });
    expect(flat).not.toHaveProperty("tilt");
  });

  it("leaves the shape itself alone where nothing changes", () => {
    expect(withImageRotation(image, imageRotation(image))).toBe(image);
  });
});
