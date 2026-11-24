// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ClearArea } from "../../../../bindings/ClearArea";

// How the screen on the main canvas is laid out on a platform's canvas of
// another shape. Nothing is cropped: the screen keeps its weight and where
// its empty space sits, read off the main canvas, and is scaled and moved
// for the platform's shape inside the part its interface leaves clear.
//
// Read: the screen's weight as the bigger of its size against the biggest it
// could be (prominence) and its share of the canvas (area); where the empty
// space sits on each axis, snapped to the middle or an edge when close; the
// padding to the edges it does not touch; and which edges it runs past.
//
// Lay out: sized by the weight inside the clear area less the padding; an
// edge touched on the main canvas stays touched where it is half of a full
// bleed or the screen still spans the other way, and an edge run past always
// does, the part past it running on past the clear area's edge. On a
// platform with a clear area a screen not stuck to the top or bottom leans
// halfway to the middle, away from the app's bars.

type Size = { height: number; width: number };
export type ScreenBox = { height: number; width: number; x: number; y: number };

const EDGES = ["left", "right", "top", "bottom"] as const;
type Edge = (typeof EDGES)[number];
type Edges = Record<Edge, boolean>;

const OPPOSITE: Record<Edge, Edge> = {
  bottom: "top",
  left: "right",
  right: "left",
  top: "bottom",
};

/** Within this share of the middle or an edge counts as there. */
const SNAP = 0.04;

/** The padding kept to edges the screen does not touch, as a share of the
 * canvas's shorter side: what the main canvas keeps, within these. */
const PADDING = { least: 0.03, most: 0.08 };

/** What the main canvas says about the screen on it. */
export type ScreenIntent = {
  /** Where the empty space sits across and down, zero to one. */
  across: number;
  /** The share of the canvas the part on it covers. */
  area: number;
  aspect: number;
  /** Where the part on the canvas sits in the whole screen, as shares. */
  cut: ScreenBox;
  /** The edges the screen runs past. */
  cutEdges: Edges;
  down: number;
  /** The smallest gap to an edge it does not touch, as a share of the
   * canvas's shorter side. */
  padding: number;
  /** Its width against the widest it could be on the canvas. */
  prominence: number;
  touches: Edges;
};

const fitted = (room: Size, aspect: number) => {
  const width = Math.min(room.width, room.height * aspect);
  return { height: width / aspect, width };
};

const snapped = (share: number) => {
  if (Math.abs(share - 0.5) < SNAP) return 0.5;
  if (Math.abs(share) < SNAP) return 0;
  if (Math.abs(share - 1) < SNAP) return 1;
  return share;
};

/** What `box`, on a canvas `canvas` big, says about where it belongs. */
export function screenIntent(canvas: Size, box: ScreenBox): ScreenIntent {
  const x = Math.max(0, box.x);
  const y = Math.max(0, box.y);
  const seen = {
    height: Math.max(1, Math.min(canvas.height, box.y + box.height) - y),
    width: Math.max(1, Math.min(canvas.width, box.x + box.width) - x),
    x,
    y,
  };
  const short = Math.min(canvas.width, canvas.height);
  const gaps: Record<Edge, number> = {
    bottom: canvas.height - seen.y - seen.height,
    left: seen.x,
    right: canvas.width - seen.x - seen.width,
    top: seen.y,
  };
  const touches = edgesWhere((edge) => gaps[edge] < short * SNAP);
  const open = EDGES.filter((edge) => !touches[edge]).map((e) => gaps[e]);
  const freeX = canvas.width - seen.width;
  const freeY = canvas.height - seen.height;
  const aspect = seen.width / seen.height;
  return {
    across: Math.abs(freeX) < 1 ? 0.5 : snapped(seen.x / freeX),
    area: (seen.width * seen.height) / (canvas.width * canvas.height),
    aspect,
    cut: {
      height: seen.height / box.height,
      width: seen.width / box.width,
      x: (seen.x - box.x) / box.width,
      y: (seen.y - box.y) / box.height,
    },
    cutEdges: {
      bottom: box.y + box.height > canvas.height + 0.5,
      left: box.x < -0.5,
      right: box.x + box.width > canvas.width + 0.5,
      top: box.y < -0.5,
    },
    down: Math.abs(freeY) < 1 ? 0.5 : snapped(seen.y / freeY),
    padding: Math.min(
      PADDING.most,
      Math.max(
        PADDING.least,
        open.length ? Math.min(...open) / short : PADDING.least,
      ),
    ),
    prominence: seen.width / fitted(canvas, aspect).width,
    touches,
  };
}

const edgesWhere = (test: (edge: Edge) => boolean): Edges => ({
  bottom: test("bottom"),
  left: test("left"),
  right: test("right"),
  top: test("top"),
});

/** The screen `intent` describes, laid out on a canvas `canvas` big whose
 * platform covers `clear`: the whole screen, the part that runs past an
 * edge included. */
export function laidScreen(
  canvas: Size,
  intent: ScreenIntent,
  clear?: ClearArea,
): ScreenBox {
  const covered = clear !== undefined && (clear.top > 0 || clear.bottom > 0);
  const area = {
    height: canvas.height * (1 - (clear?.top ?? 0) - (clear?.bottom ?? 0)),
    width: canvas.width,
    x: 0,
    y: canvas.height * (clear?.top ?? 0),
  };
  const pad = intent.padding * Math.min(canvas.width, canvas.height);
  const sideMargin = (clear?.side ?? 0) * canvas.width;
  const side = Math.max(pad, sideMargin);
  // What the weight is measured against: the clear area, less the rail's
  // margin on each side.
  const inner = { height: area.height, width: area.width - sideMargin * 2 };
  const place = (held: Edges) => {
    const inset = (edge: Edge) =>
      held[edge] ? 0 : edge === "left" || edge === "right" ? side : pad;
    const room = {
      height: area.height - inset("top") - inset("bottom"),
      width: area.width - inset("left") - inset("right"),
      x: area.x + inset("left"),
      y: area.y + inset("top"),
    };
    const byFit = fitted(inner, intent.aspect).width * intent.prominence;
    const byArea = Math.sqrt(
      intent.area * inner.width * inner.height * intent.aspect,
    );
    const width = Math.min(
      fitted(room, intent.aspect).width,
      Math.max(byFit, byArea),
    );
    return { height: width / intent.aspect, room, width };
  };
  let held = intent.touches;
  let fit = place(held);
  const spans = {
    x: fit.width >= fit.room.width - 0.5,
    y: fit.height >= fit.room.height - 0.5,
  };
  const keep = (edge: Edge) =>
    intent.cutEdges[edge] ||
    (held[edge] &&
      (intent.touches[OPPOSITE[edge]] ||
        (edge === "left" || edge === "right" ? spans.y : spans.x)));
  const next = edgesWhere(keep);
  if (EDGES.some((edge) => next[edge] !== held[edge])) {
    held = next;
    fit = place(held);
  }
  const { height, room, width } = fit;
  const down =
    covered && !held.top && !held.bottom
      ? 0.5 + (intent.down - 0.5) / 2
      : intent.down;
  const seen = {
    x: room.x + intent.across * (room.width - width),
    y: room.y + down * (room.height - height),
  };
  const whole = {
    height: height / intent.cut.height,
    width: width / intent.cut.width,
  };
  return {
    ...whole,
    x: seen.x - intent.cut.x * whole.width,
    y: seen.y - intent.cut.y * whole.height,
  };
}
