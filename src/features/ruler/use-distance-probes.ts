// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useRef, useState } from "react";

import { combineDistanceProbes } from "./distance-probe-range";
import { Axis, GradientField } from "./gradient-field";
import {
  distanceProbeAt,
  DistanceProbe,
  PixelSize,
  Point,
} from "./pixel-analysis";
import { clipProbe } from "./probe-stops";
import { Guide, Measurement } from "./ruler-types";

export type PersistedDistanceProbe = DistanceProbe & { id: number };

/** Committed lines the probes should also stop at; only set while Alt is held. */
export type ProbeArtifacts = {
  guides: readonly Guide[];
  measurements: readonly Measurement[];
};

export function useDistanceProbes({
  artifacts,
  cursor,
  field,
  threshold,
  viewport,
}: {
  threshold: number;
  viewport: PixelSize;
  artifacts?: ProbeArtifacts;
  cursor?: Point;
  field?: GradientField;
}) {
  const [probes, setProbes] = useState<PersistedDistanceProbe[]>([]);
  const nextIdRef = useRef(1);
  const { height: viewportHeight, width: viewportWidth } = viewport;
  const artifactGuides = artifacts?.guides;
  const artifactMeasurements = artifacts?.measurements;
  const previews =
    cursor && field
      ? (["x", "y"] as const).map((axis) => {
          const probe = distanceProbeAt({
            axis,
            field,
            point: cursor,
            threshold,
            viewport,
          });
          return artifacts ? clipProbe({ ...artifacts, cursor, probe }) : probe;
        })
      : [];
  const between = useCallback(
    (axis: Axis, start: Point, end: Point) => {
      if (!field) return undefined;
      const shared = {
        axis,
        field,
        threshold,
        viewport: { height: viewportHeight, width: viewportWidth },
      };
      // The initial along-axis position remains the range anchor, while both
      // ends are re-sampled on the cursor's current scanline so the live ruler
      // follows perpendicular pointer movement before it is stamped.
      const trackingStart =
        axis === "x" ? { x: start.x, y: end.y } : { x: end.x, y: start.y };
      const startProbe = distanceProbeAt({ ...shared, point: trackingStart });
      const endProbe = distanceProbeAt({ ...shared, point: end });
      const combined = combineDistanceProbes({
        endPoint: end,
        endProbe,
        startPoint: trackingStart,
        startProbe,
      });
      const sameAlong = axis === "x" ? start.x === end.x : start.y === end.y;
      return artifactGuides && artifactMeasurements && sameAlong
        ? clipProbe({
            cursor: end,
            guides: artifactGuides,
            measurements: artifactMeasurements,
            probe: combined,
          })
        : combined;
    },
    [
      artifactGuides,
      artifactMeasurements,
      field,
      threshold,
      viewportHeight,
      viewportWidth,
    ],
  );
  const persistProbe = useCallback((probe: DistanceProbe) => {
    setProbes((current) => [...current, { ...probe, id: nextIdRef.current++ }]);
  }, []);
  const clear = useCallback(() => {
    setProbes([]);
  }, []);
  const remove = useCallback((id: number) => {
    setProbes((current) => current.filter((probe) => probe.id !== id));
  }, []);
  /** Undo/redo puts a whole list back; the id counter only ever climbs. */
  const restore = useCallback((next: PersistedDistanceProbe[]) => {
    setProbes(next);
  }, []);
  return {
    between,
    clear,
    persistProbe,
    previews,
    probes,
    remove,
    restore,
  };
}
