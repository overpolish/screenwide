// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Axis } from "./gradient-field";
import { DistanceProbe, Point } from "./pixel-analysis";

const alongOf = (axis: Axis, point: Point) =>
  axis === "x" ? point.x : point.y;

/**
 * Joins the locally detected ruler at each end of a drag. Walking forward takes
 * the first ruler's near edge and the last ruler's far edge; walking backward
 * mirrors that choice. Any intervening borders therefore remain inside one
 * continuous measurement instead of splitting it into adjacent rulers.
 */
export const combineDistanceProbes = ({
  endPoint,
  endProbe,
  startPoint,
  startProbe,
}: {
  endPoint: Point;
  endProbe: DistanceProbe;
  startPoint: Point;
  startProbe: DistanceProbe;
}): DistanceProbe => {
  const forward =
    alongOf(startProbe.axis, endPoint) >= alongOf(startProbe.axis, startPoint);
  return {
    axis: startProbe.axis,
    end: forward ? endProbe.end : startProbe.end,
    position: startProbe.position,
    start: forward ? startProbe.start : endProbe.start,
  };
};
