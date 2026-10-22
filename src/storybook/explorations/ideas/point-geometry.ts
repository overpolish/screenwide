// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ACTIONS, type Element, ELEMENTS } from "./ideas-data";

// Reading the picture as an interface: which element is under a point, where
// its nearest edge is, and when it was used.

export const W = 720;
export const H = 450;
/** How close an arrow's head must come to an edge to find it, in px. */
export const SNAP = 44;

export const spring = { bounce: 0.2, duration: 0.35, type: "spring" } as const;

export const px = ({ box }: Element) => ({
  h: (box.h / 100) * H,
  w: (box.w / 100) * W,
  x: (box.x / 100) * W,
  y: (box.y / 100) * H,
});

/** The smallest element under a point, so a button wins over its card. */
export const elementAt = (x: number, y: number) =>
  ELEMENTS.map((element) => ({ element, rect: px(element) }))
    .filter(
      ({ rect }) =>
        x >= rect.x &&
        x <= rect.x + rect.w &&
        y >= rect.y &&
        y <= rect.y + rect.h,
    )
    .sort(
      (first, second) =>
        first.rect.w * first.rect.h - second.rect.w * second.rect.h,
    )
    .map(({ element }) => element)[0] as Element | undefined;

/** The nearest point on an element's edge, and how far it is. */
const edgePoint = (element: Element, x: number, y: number) => {
  const rect = px(element);
  const cx = Math.min(Math.max(x, rect.x), rect.x + rect.w);
  const cy = Math.min(Math.max(y, rect.y), rect.y + rect.h);
  const inside = cx === x && cy === y;
  const point = inside
    ? { x: x < rect.x + rect.w / 2 ? rect.x : rect.x + rect.w, y }
    : { x: cx, y: cy };
  return { distance: Math.hypot(point.x - x, point.y - y), point };
};

export const nearestEdge = (x: number, y: number) =>
  ELEMENTS.map((element) => ({ element, ...edgePoint(element, x, y) })).sort(
    (first, second) => first.distance - second.distance,
  )[0];

/** The recorded action on an element, which says when to show what points at it. */
export const appearsAt = (element: Element) =>
  ACTIONS.find((action) => action.element === element.id);
