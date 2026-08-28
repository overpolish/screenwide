// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { RulerComponentBox } from "./api";
import { cornerRadiusAt } from "./corner-radius";
import { GradientField } from "./gradient-field";
import { Corner } from "./ruler-types";

const emptyField = (): GradientField => {
  const width = 80;
  const height = 80;
  return {
    colSum: new Float32Array(width),
    gx: new Uint8Array(width * height),
    gy: new Uint8Array(width * height),
    height,
    rowSum: new Float32Array(height),
    width,
  };
};

const localPixel = ({
  box,
  corner,
  u,
  v,
}: {
  box: RulerComponentBox;
  corner: Corner;
  u: number;
  v: number;
}) => ({
  x: corner.endsWith("right") ? box.x + box.width - u : box.x + u,
  y: corner.startsWith("bottom") ? box.y + box.height - v : box.y + v,
});

const paintCorner = ({
  box,
  corner = "top-left",
  field,
  radius,
  strength,
}: {
  box: RulerComponentBox;
  field: GradientField;
  radius: number;
  strength: number;
  corner?: Corner;
}) => {
  for (let step = 0; step <= 90; step += 1) {
    const angle = (step / 90) * (Math.PI / 2);
    const { x, y } = localPixel({
      box,
      corner,
      u: Math.round(radius - radius * Math.cos(angle)),
      v: Math.round(radius - radius * Math.sin(angle)),
    });
    const index = y * field.width + x;
    field.gx[index] = Math.round(strength * Math.cos(angle));
    field.gy[index] = Math.round(strength * Math.sin(angle));
  }
  for (let v = radius; v <= box.height; v += 1) {
    const { x, y } = localPixel({ box, corner, u: 0, v });
    field.gx[y * field.width + x] = strength;
  }
  for (let u = radius; u <= box.width; u += 1) {
    const { x, y } = localPixel({ box, corner, u, v: 0 });
    field.gy[y * field.width + x] = strength;
  }
};

const box = { height: 40, width: 50, x: 10, y: 10 };
const viewport = { height: 80, width: 80 };

describe("cornerRadiusAt", () => {
  it("fits the continuous curve rather than only its tangencies", () => {
    const field = emptyField();
    paintCorner({ box, field, radius: 8, strength: 30 });

    expect(
      cornerRadiusAt({
        boxes: [box],
        cursor: { x: 12, y: 12 },
        field,
        threshold: 24,
        viewport,
      }),
    ).toEqual({ box, confidence: "high", corner: "top-left", radius: 8 });
  });

  it("uses the active tolerance threshold", () => {
    const field = emptyField();
    paintCorner({ box, field, radius: 8, strength: 10 });
    expect(
      cornerRadiusAt({
        boxes: [box],
        cursor: { x: 12, y: 12 },
        field,
        threshold: 24,
        viewport,
      }),
    ).toBeUndefined();
    expect(
      cornerRadiusAt({
        boxes: [box],
        cursor: { x: 12, y: 12 },
        field,
        threshold: 5,
        viewport,
      })?.radius,
    ).toBe(8);
  });

  it("selects the nested curve whose fitted arc is nearest the cursor", () => {
    const field = emptyField();
    const outer = { height: 54, width: 60, x: 8, y: 8 };
    const inner = { height: 42, width: 48, x: 14, y: 14 };
    paintCorner({ box: outer, field, radius: 12, strength: 40 });
    paintCorner({ box: inner, field, radius: 6, strength: 40 });

    expect(
      cornerRadiusAt({
        boxes: [inner, outer],
        cursor: { x: 12, y: 12 },
        field,
        threshold: 24,
        viewport,
      }),
    ).toMatchObject({ box: outer, radius: 12 });
    expect(
      cornerRadiusAt({
        boxes: [outer, inner],
        cursor: { x: 16, y: 16 },
        field,
        threshold: 24,
        viewport,
      }),
    ).toMatchObject({ box: inner, radius: 6 });
  });

  it("fits every corner orientation", () => {
    for (const corner of [
      "top-left",
      "top-right",
      "bottom-left",
      "bottom-right",
    ] as const) {
      const field = emptyField();
      paintCorner({ box, corner, field, radius: 8, strength: 30 });
      const virtual = localPixel({ box, corner, u: 0, v: 0 });
      const cursor = localPixel({ box, corner, u: 2, v: 2 });
      expect(
        cornerRadiusAt({
          boxes: [box],
          cursor,
          field,
          threshold: 24,
          viewport,
        }),
        `${corner} at ${String(virtual.x)},${String(virtual.y)}`,
      ).toMatchObject({ corner, radius: 8 });
    }
  });

  it("does not preview a valid corner when the cursor is far from its arc", () => {
    const field = emptyField();
    paintCorner({ box, field, radius: 8, strength: 30 });
    expect(
      cornerRadiusAt({
        boxes: [box],
        cursor: { x: 50, y: 45 },
        field,
        threshold: 24,
        viewport,
      }),
    ).toBeUndefined();
  });
});
