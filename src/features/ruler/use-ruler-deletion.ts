// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Dispatch, SetStateAction, useCallback, useRef } from "react";

import { Point } from "./pixel-analysis";
import { radiusGeometry } from "./radius-geometry";
import {
  DistanceProbe,
  Guide,
  Measurement,
  RadiusMeasurement,
} from "./ruler-types";

/** Cursor-to-line distance, in screen px, that counts as "selected". */
const SELECT_RADIUS = 6;

/** Trailing ids in a label key: `m3`, `p7`, `r4`, `gx:2-5`. */
const labelIds = (key: string) =>
  key
    .slice(1)
    .split(/[:-]/u)
    .map(Number)
    .filter((value) => Number.isFinite(value));

export type SelectedLine = {
  id: number;
  kind: "guide" | "measurement" | "probe" | "radius";
};

/** The line a hovered label chip stands for - what deleting it would remove. */
export const selectedFromLabel = (key: string): SelectedLine => {
  const ids = labelIds(key);
  if (key.startsWith("m")) return { id: ids[0], kind: "measurement" };
  if (key.startsWith("p")) return { id: ids[0], kind: "probe" };
  if (key.startsWith("r")) return { id: ids[0], kind: "radius" };
  // A gap chip deletes the later-placed of its two guides.
  return { id: Math.max(...ids), kind: "guide" };
};

/**
 * Everything the delete key can act on: a hovered label chip (which owns its
 * measurement, probe, or gap), or the guide/probe line nearest the cursor.
 * Call `selectLine` during render once the placement state is known - the
 * chosen line wears a pulsing halo, and the keydown handler reads the same
 * choice through a ref so the key always deletes what the halo shows.
 */
export function useRulerDeletion({
  clearHover,
  guides,
  hovered,
  measurements,
  probes,
  radii,
  record,
  removeGuide,
  removeProbe,
  setMeasurements,
  setRadii,
  zoom,
}: {
  clearHover: () => void;
  guides: readonly Guide[];
  hovered: string | undefined;
  measurements: readonly Measurement[];
  probes: readonly DistanceProbe[];
  radii: readonly RadiusMeasurement[];
  /** Snapshots the document; only called once a deletion is certain. */
  record: () => void;
  removeGuide: (id: number) => void;
  removeProbe: (id: number) => void;
  setMeasurements: Dispatch<SetStateAction<Measurement[]>>;
  setRadii: Dispatch<SetStateAction<RadiusMeasurement[]>>;
  zoom: number;
}) {
  const selectedRef = useRef<SelectedLine | undefined>(undefined);

  const selectLine = ({
    active,
    cursor,
  }: {
    active: boolean;
    cursor?: Point;
  }): SelectedLine | undefined => {
    let best: SelectedLine | undefined;
    if (active && cursor) {
      let bestDistance = SELECT_RADIUS / Math.max(0.01, zoom);
      for (const guide of guides) {
        const distance = Math.abs(
          (guide.axis === "x" ? cursor.x : cursor.y) - guide.position,
        );
        if (distance <= bestDistance) {
          best = { id: guide.id, kind: "guide" };
          bestDistance = distance;
        }
      }
      for (const probe of probes) {
        if (probe.id === undefined) continue;
        // A probe measures ALONG its axis: its line runs from start to end on
        // that axis, sitting at `position` on the other one.
        const along = probe.axis === "x" ? cursor.x : cursor.y;
        const cross = probe.axis === "x" ? cursor.y : cursor.x;
        if (along < Math.min(probe.start, probe.end) - bestDistance) continue;
        if (along > Math.max(probe.start, probe.end) + bestDistance) continue;
        const distance = Math.abs(cross - probe.position);
        if (distance <= bestDistance) {
          best = { id: probe.id, kind: "probe" };
          bestDistance = distance;
        }
      }
      for (const measurement of measurements) {
        // Distance to the nearest point of the box OUTLINE (not its interior),
        // so a box only lights up when the cursor rides its border.
        const dx = Math.max(
          measurement.x - cursor.x,
          0,
          cursor.x - (measurement.x + measurement.width),
        );
        const dy = Math.max(
          measurement.y - cursor.y,
          0,
          cursor.y - (measurement.y + measurement.height),
        );
        const outside = Math.hypot(dx, dy);
        const inside = Math.min(
          cursor.x - measurement.x,
          measurement.x + measurement.width - cursor.x,
          cursor.y - measurement.y,
          measurement.y + measurement.height - cursor.y,
        );
        const distance = outside > 0 ? outside : Math.max(0, inside);
        if (distance <= bestDistance) {
          best = { id: measurement.id, kind: "measurement" };
          bestDistance = distance;
        }
      }
      for (const radius of radii) {
        if (radius.id === undefined) continue;
        const geometry = radiusGeometry(radius);
        const signX = radius.corner.endsWith("right") ? 1 : -1;
        const signY = radius.corner.startsWith("bottom") ? 1 : -1;
        const dx = cursor.x - geometry.center.x;
        const dy = cursor.y - geometry.center.y;
        if (dx * signX < -bestDistance || dy * signY < -bestDistance) continue;
        const distance = Math.abs(Math.hypot(dx, dy) - radius.radius);
        if (distance <= bestDistance) {
          best = { id: radius.id, kind: "radius" };
          bestDistance = distance;
        }
      }
    }
    selectedRef.current = best;
    return best;
  };

  const deleteHovered = useCallback(() => {
    if (hovered) {
      const ids = labelIds(hovered);
      record();
      if (hovered.startsWith("m"))
        setMeasurements((current) =>
          current.filter((measurement) => measurement.id !== ids[0]),
        );
      else if (hovered.startsWith("p")) removeProbe(ids[0]);
      else if (hovered.startsWith("r"))
        setRadii((current) => current.filter((radius) => radius.id !== ids[0]));
      // Dropping the later-placed guide merges the neighbouring gaps.
      else removeGuide(Math.max(...ids));
      clearHover();
      return true;
    }
    const selected = selectedRef.current;
    if (selected === undefined) return false;
    record();
    if (selected.kind === "guide") removeGuide(selected.id);
    else if (selected.kind === "probe") removeProbe(selected.id);
    else if (selected.kind === "radius")
      setRadii((current) =>
        current.filter((radius) => radius.id !== selected.id),
      );
    else
      setMeasurements((current) =>
        current.filter((measurement) => measurement.id !== selected.id),
      );
    return true;
  }, [
    clearHover,
    hovered,
    record,
    removeGuide,
    removeProbe,
    setMeasurements,
    setRadii,
  ]);

  return { deleteHovered, selectLine };
}
