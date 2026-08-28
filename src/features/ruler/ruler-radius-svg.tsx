// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Bounds } from "./pixel-analysis";
import { radiusGeometry, radiusLabelPlacement } from "./radius-geometry";
import { LABEL_HEIGHT, labelWidth } from "./ruler-label-metrics";
import { SvgLabel } from "./ruler-svg-label";
import { RadiusMeasurement } from "./ruler-types";
import { LabelHandles } from "./use-label-handles";

export function RadiusMeasurementSvg({
  handles,
  measurement,
  selected,
  showLabel = true,
  showShape = true,
  visibleBounds,
}: {
  measurement: RadiusMeasurement;
  handles?: LabelHandles;
  selected?: boolean;
  showLabel?: boolean;
  showShape?: boolean;
  visibleBounds?: Bounds;
}) {
  const geometry = radiusGeometry(measurement);
  const labelKey =
    measurement.id === undefined ? undefined : `r${String(measurement.id)}`;
  const text = `${measurement.confidence === "low" ? "≈ " : ""}${String(Math.round(measurement.radius))} px`;
  const placement = radiusLabelPlacement({
    geometry,
    labelSize: { height: LABEL_HEIGHT, width: labelWidth(text) },
    measurement,
    visibleBounds,
  });
  const offset =
    handles && labelKey ? handles.offset(labelKey) : { x: 0, y: 0 };
  const leaderEnd = {
    x: placement.leaderEnd.x + offset.x,
    y: placement.leaderEnd.y + offset.y,
  };
  const leaderLength = Math.hypot(
    leaderEnd.x - geometry.arcMidpoint.x,
    leaderEnd.y - geometry.arcMidpoint.y,
  );
  return (
    <g className="stroke-error">
      {showShape ? (
        <>
          <path
            className={
              selected
                ? "animate-halo transition-opacity duration-75"
                : "transition-opacity duration-75"
            }
            d={geometry.path}
            fill="none"
            opacity={selected ? 0.4 : 0}
            strokeWidth={7}
            vectorEffect="non-scaling-stroke"
          />
          <path
            d={geometry.path}
            fill="none"
            opacity={measurement.confidence === "low" ? 0.7 : 1}
            strokeDasharray={
              measurement.confidence === "low" ? "4 3" : undefined
            }
            strokeWidth={2}
            vectorEffect="non-scaling-stroke"
          />
          <line
            vectorEffect="non-scaling-stroke"
            x1={geometry.center.x}
            x2={geometry.arcMidpoint.x}
            y1={geometry.center.y}
            y2={geometry.arcMidpoint.y}
          />
        </>
      ) : null}
      {showLabel ? (
        <>
          {leaderLength > 4 ? (
            <line
              opacity={0.45}
              strokeDasharray="2 3"
              vectorEffect="non-scaling-stroke"
              x1={geometry.arcMidpoint.x}
              x2={leaderEnd.x}
              y1={geometry.arcMidpoint.y}
              y2={leaderEnd.y}
            />
          ) : null}
          <SvgLabel
            handles={handles}
            labelKey={labelKey}
            text={text}
            x={placement.x}
            y={placement.y}
          />
        </>
      ) : null}
    </g>
  );
}
