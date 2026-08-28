// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CSSProperties } from "react";

import { Bounds, PixelSize } from "./pixel-analysis";
import { GuideGapLabels } from "./ruler-guide-gaps";
import { DistanceProbeSvg } from "./ruler-probe-svg";
import { RadiusMeasurementSvg } from "./ruler-radius-svg";
import { MeasurementSvg } from "./ruler-svg-overlay";
import {
  DistanceProbe,
  Guide,
  Measurement,
  RadiusMeasurement,
} from "./ruler-types";
import { LabelHandles } from "./use-label-handles";
import { useSettleAnimation } from "./use-settle-animation";

/** All persisted labels share one final layer above every world-space stroke. */
export function RulerLabelLayer({
  guides,
  handles,
  measurements,
  probes,
  radii,
  radiusPreview,
  style,
  viewport,
  visibleBounds,
}: {
  guides: readonly Guide[];
  handles: LabelHandles;
  measurements: readonly Measurement[];
  probes: readonly DistanceProbe[];
  radii: readonly RadiusMeasurement[];
  style: CSSProperties;
  viewport: PixelSize;
  radiusPreview?: RadiusMeasurement;
  visibleBounds?: Bounds;
}) {
  const frames = useSettleAnimation(measurements);
  const settled = measurements.map((measurement) => {
    const frame = frames.get(measurement.id);
    return frame ? { ...measurement, ...frame } : measurement;
  });
  return (
    <div className="pointer-events-none absolute inset-0" style={style}>
      <svg className="pointer-events-none absolute inset-0 size-full overflow-visible">
        {probes.map((probe) => (
          <DistanceProbeSvg
            handles={handles}
            key={probe.id}
            probe={probe}
            showLabel={handles.isVisible(`p${String(probe.id)}`)}
            showLine={false}
          />
        ))}
        {radii.map((radius) => (
          <RadiusMeasurementSvg
            handles={handles}
            key={radius.id}
            measurement={radius}
            showLabel={handles.isVisible(`r${String(radius.id)}`)}
            showShape={false}
            visibleBounds={visibleBounds}
          />
        ))}
        {radiusPreview ? (
          <RadiusMeasurementSvg
            measurement={radiusPreview}
            showShape={false}
            visibleBounds={visibleBounds}
          />
        ) : null}
        {settled.map((measurement) => (
          <MeasurementSvg
            handles={handles}
            key={measurement.id}
            measurement={measurement}
            showLabel={handles.isVisible(`m${String(measurement.id)}`)}
            showShape={false}
          />
        ))}
        <GuideGapLabels guides={guides} handles={handles} viewport={viewport} />
      </svg>
    </div>
  );
}
