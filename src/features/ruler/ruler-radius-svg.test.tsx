// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { radiusGeometry, radiusLabelPlacement } from "./radius-geometry";
import { RadiusMeasurementSvg } from "./ruler-radius-svg";
import { RadiusMeasurement } from "./ruler-types";

const topLeft: RadiusMeasurement = {
  confidence: "high",
  corner: "top-left",
  height: 40,
  radius: 8,
  width: 60,
  x: 10,
  y: 20,
};

describe("radiusGeometry", () => {
  it("places the fitted arc and centre inside the selected corner", () => {
    expect(radiusGeometry(topLeft)).toMatchObject({
      arcEnd: { x: 18, y: 20 },
      arcStart: { x: 10, y: 28 },
      center: { x: 18, y: 28 },
    });
  });

  it("winds each corner between its correct edge tangencies", () => {
    const cases = [
      {
        arcEnd: { x: 18, y: 20 },
        arcStart: { x: 10, y: 28 },
        corner: "top-left",
      },
      {
        arcEnd: { x: 70, y: 28 },
        arcStart: { x: 62, y: 20 },
        corner: "top-right",
      },
      {
        arcEnd: { x: 10, y: 52 },
        arcStart: { x: 18, y: 60 },
        corner: "bottom-left",
      },
      {
        arcEnd: { x: 62, y: 60 },
        arcStart: { x: 70, y: 52 },
        corner: "bottom-right",
      },
    ] as const;

    for (const { arcEnd, arcStart, corner } of cases)
      expect(radiusGeometry({ ...topLeft, corner })).toMatchObject({
        arcEnd,
        arcStart,
      });
  });

  it("parks labels outside the component and flips at viewport edges", () => {
    const nearEdge = { ...topLeft, x: 5, y: 5 };
    const geometry = radiusGeometry(nearEdge);
    const placement = radiusLabelPlacement({
      geometry,
      labelSize: { height: 20, width: 40 },
      measurement: nearEdge,
      visibleBounds: { height: 100, width: 120, x: 0, y: 0 },
    });
    const labelLeft = placement.x - 20;
    const labelTop = placement.y - 10;
    const outside =
      labelLeft >= nearEdge.x + nearEdge.width ||
      labelLeft + 40 <= nearEdge.x ||
      labelTop >= nearEdge.y + nearEdge.height ||
      labelTop + 20 <= nearEdge.y;

    expect(outside).toBe(true);
    expect(placement.leaderEnd).not.toEqual(geometry.arcMidpoint);
  });
});

describe("RadiusMeasurementSvg", () => {
  it("marks low-confidence radius labels as estimates", () => {
    const markup = renderToStaticMarkup(
      <svg>
        <RadiusMeasurementSvg measurement={{ ...topLeft, confidence: "low" }} />
      </svg>,
    );

    expect(markup).toContain("≈ 8 px");
    expect(markup).toContain('stroke-dasharray="4 3"');
  });
});
