// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  cancelFrame,
  frame,
  type MotionValue,
  motionValue,
} from "motion/react";
import { type RefObject, useLayoutEffect, useState } from "react";

import { islandOutline } from "./island-outline";

// The values the island's parts are animated by, and the drawing of its
// outline from them. Motion animates these values where it animated the
// parts' own styles, so every transition stays as it is; the outline is
// redrawn from them once in each frame that any of them changes, in Motion's
// render step, so it is never a frame behind the parts it outlines, and not
// at all while the island is still.

export type IslandShapeValues = ReturnType<typeof createShapeValues>;

const createShapeValues = (corners: {
  flare: number;
  radius: number;
  top: number;
}) => ({
  flare: motionValue(corners.flare),
  radius: motionValue(corners.radius),
  row: {
    height: motionValue(0),
    opacity: motionValue(1),
  },
  section: {
    height: motionValue(0),
    opacity: motionValue(0),
    /** From the island's middle, as the section is placed. */
    start: motionValue("0px"),
    top: motionValue(0),
    width: motionValue(0),
  },
  top: motionValue(corners.top),
});

/** The island's shape values, made once, starting from its first corners. */
export function useIslandShapeValues(corners: {
  flare: number;
  radius: number;
  top: number;
}) {
  const [values] = useState(() => createShapeValues(corners));
  return values;
}

const read = (value: MotionValue<number> | MotionValue<string>) =>
  Number.parseFloat(String(value.get())) || 0;

/** Draws the outline into `shape`, and, while the section fades on its own
 * as the island comes out or goes away, the row alone into `row` beneath it:
 * one path cannot fade only part of itself. */
export function useIslandOutline({
  rowPathRef,
  shapePathRef,
  values,
  width,
}: {
  rowPathRef: RefObject<SVGPathElement | null>;
  shapePathRef: RefObject<SVGPathElement | null>;
  values: IslandShapeValues;
  width: MotionValue<number>;
}) {
  useLayoutEffect(() => {
    const draw = () => {
      const shape = shapePathRef.current;
      const under = rowPathRef.current;
      if (!shape || !under) return;
      const rowWidth = width.get();
      const row = {
        flare: read(values.flare),
        height: read(values.row.height),
        radius: read(values.radius),
        top: read(values.top),
        width: rowWidth,
      };
      const rowOpacity = read(values.row.opacity);
      const sectionOpacity = read(values.section.opacity);
      const shown =
        read(values.section.top) + read(values.section.height) - row.height;
      const section =
        sectionOpacity > 0.001 && shown >= 0.5
          ? {
              height: shown,
              start: rowWidth / 2 + read(values.section.start),
              width: read(values.section.width),
            }
          : null;
      const isWhole = !section || sectionOpacity >= 0.999;
      shape.setAttribute("d", islandOutline({ row, section }));
      shape.style.opacity = String(
        isWhole ? rowOpacity : rowOpacity * sectionOpacity,
      );
      under.setAttribute(
        "d",
        isWhole ? "" : islandOutline({ row, section: null }),
      );
      under.style.opacity = String(rowOpacity);
    };
    let isQueued = false;
    const render = () => {
      isQueued = false;
      draw();
    };
    const queue = () => {
      if (isQueued) return;
      isQueued = true;
      frame.render(render);
    };
    draw();
    const stops = [
      width,
      values.flare,
      values.radius,
      values.top,
      values.row.height,
      values.row.opacity,
      ...Object.values(values.section),
    ].map((value) => (value as MotionValue<unknown>).on("change", queue));
    return () => {
      stops.forEach((stop) => {
        stop();
      });
      cancelFrame(render);
    };
  }, [rowPathRef, shapePathRef, values, width]);
}
