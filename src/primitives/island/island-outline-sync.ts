// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  animate,
  cancelFrame,
  frame,
  type MotionValue,
  motionValue,
} from "motion/react";
import { type RefObject, useLayoutEffect, useRef, useState } from "react";

import { ISLAND } from "./island-geometry";
import { glide } from "./island-motion";
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

/** The island's shape values, made once, starting from its first corners,
 * and its corners glided to `corners` as they change. */
export function useIslandShapeValues(corners: {
  flare: number;
  radius: number;
  top: number;
}) {
  const [values] = useState(() => createShapeValues(corners));
  const { flare, radius, top } = corners;
  useLayoutEffect(() => {
    const gliding = [
      animate(values.flare, flare, glide),
      animate(values.radius, radius, glide),
      animate(values.top, top, glide),
    ];
    return () => {
      for (const corner of gliding) corner.stop();
    };
  }, [flare, radius, top, values]);
  return values;
}

/** A part of the island, in CSS pixels from the top left of the page. */
export type IslandPart = {
  height: number;
  width: number;
  x: number;
  y: number;
};

const sameParts = (a: IslandPart[], b: IslandPart[]) =>
  a.length === b.length &&
  a.every(
    (part, index) =>
      Math.abs(part.x - b[index].x) < 0.5 &&
      Math.abs(part.y - b[index].y) < 0.5 &&
      Math.abs(part.width - b[index].width) < 0.5 &&
      Math.abs(part.height - b[index].height) < 0.5,
  );

const read = (value: MotionValue<number> | MotionValue<string>) =>
  Number.parseFloat(String(value.get())) || 0;

/** Draws the outline into `shape`, and, while the section fades on its own
 * as the island comes out or goes away, the row alone into `row` beneath it:
 * one path cannot fade only part of itself. Clips `clip` to the outline, so
 * nothing the island holds shows outside the black while it changes shape:
 * what hangs is laid out at its full size at once and only revealed as the
 * black grows round it. Tells `onShapeChange` where the row and the section
 * are, as they move, and nothing while they rest. */
export function useIslandOutline({
  clipRef,
  onShapeChange,
  rowPathRef,
  shapePathRef,
  values,
  width,
  x,
}: {
  /** Laid out as the outline is drawn, from the row's start: in a
   * right-to-left page it is mirrored as the outline is. */
  clipRef: RefObject<HTMLElement | null>;
  rowPathRef: RefObject<SVGPathElement | null>;
  shapePathRef: RefObject<SVGPathElement | null>;
  values: IslandShapeValues;
  width: MotionValue<number>;
  /** How far the whole island has moved sideways, to keep the gap in its
   * row under the notch: its parts move with it. */
  x: MotionValue<number>;
  onShapeChange?: (parts: IslandPart[]) => void;
}) {
  const onShapeChangeRef = useRef(onShapeChange);
  useLayoutEffect(() => {
    onShapeChangeRef.current = onShapeChange;
  });
  useLayoutEffect(() => {
    let reported: IslandPart[] = [];
    const report = ({
      extent,
      row,
      rowOpacity,
      section,
      svg,
    }: {
      extent: { end: number; start: number };
      row: { height: number; width: number };
      rowOpacity: number;
      section: { height: number; start: number; width: number } | null;
      svg: SVGSVGElement;
    }) => {
      const onShape = onShapeChangeRef.current;
      if (!onShape) return;
      // The outline is drawn from the row's start, so in a right-to-left
      // page its parts are measured from the other side. Its box spans the
      // outline, with room round it for the shadow.
      const room = ISLAND.shadowRoom;
      const outer = svg.getBoundingClientRect();
      const isRtl = getComputedStyle(svg).direction === "rtl";
      const rowLeft = isRtl
        ? outer.left + room + extent.end - row.width
        : outer.left + room - extent.start;
      const top = outer.top + room;
      const parts: IslandPart[] = [];
      if (rowOpacity > 0.01) {
        parts.push({
          height: row.height,
          width: row.width,
          x: rowLeft,
          y: top,
        });
      }
      if (section) {
        parts.push({
          height: section.height,
          width: section.width,
          x:
            rowLeft +
            (isRtl ? row.width - section.start - section.width : section.start),
          y: top + row.height,
        });
      }
      if (sameParts(parts, reported)) return;
      reported = parts;
      onShape(parts);
    };
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
      const outline = islandOutline({ row, section });
      shape.setAttribute("d", outline);
      if (clipRef.current)
        clipRef.current.style.clipPath = `path("${outline}")`;
      shape.style.opacity = String(
        isWhole ? rowOpacity : rowOpacity * sectionOpacity,
      );
      under.setAttribute(
        "d",
        isWhole ? "" : islandOutline({ row, section: null }),
      );
      under.style.opacity = String(rowOpacity);
      const svg = shape.ownerSVGElement;
      if (svg) {
        // The box spans the outline, with the shadow's room round it, and
        // no further: a section reaching past the row is inside it, so its
        // shadow is painted, and cleared, as the rest is; and an empty box
        // below the island never adds to the page's scrolling, as it would
        // in Storybook's Docs.
        const room = ISLAND.shadowRoom;
        const extent = {
          end: Math.max(rowWidth, section ? section.start + section.width : 0),
          start: Math.min(0, section?.start ?? 0),
        };
        svg.style.height = `${String(
          row.height + (section?.height ?? 0) + room * 2,
        )}px`;
        svg.style.width = `${String(extent.end - extent.start + room * 2)}px`;
        svg.style.insetInlineStart = `${String(extent.start - room)}px`;
        shape.parentElement?.setAttribute(
          "transform",
          `translate(${String(room - extent.start)} ${String(room)})`,
        );
        report({ extent, row, rowOpacity, section, svg });
      }
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
      x,
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
  }, [clipRef, rowPathRef, shapePathRef, values, width, x]);
}
