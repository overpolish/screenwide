// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Element, elementById, ELEMENTS } from "./ideas-data";
import { type LookId, type Pen, type Size, type Swatch } from "./looks";
import { H, px, W } from "./point-geometry";

// The marks on the picture and where each one's choices open.

export type Point = { x: number; y: number };

export type Mark = {
  element: Element;
  id: number;
  lookOverride: LookId | null;
  pen: Pen;
  size: Size;
  /** Unset: the look's default, ink for Product and the accent otherwise. */
  swatch: Swatch | null;
  from?: Point;
  to?: Point;
};

export const newMark = ({
  element,
  id,
  pen,
  ...rest
}: Pick<Mark, "element" | "id" | "pen"> & Partial<Mark>): Mark => ({
  element,
  id,
  lookOverride: null,
  pen,
  size: "medium",
  swatch: null,
  ...rest,
});

const seeded = (id: string) => elementById(id) ?? ELEMENTS[0];

export const SEEDED: Mark[] = [
  newMark({ element: seeded("starter"), id: 1, pen: "focus" }),
  newMark({ element: seeded("new"), id: 2, pen: "counter" }),
  newMark({
    element: seeded("share"),
    from: { x: 520, y: 120 },
    id: 3,
    pen: "arrow",
    to: { x: 634, y: 28 },
  }),
];

/** The corner of an element that faces the middle of the picture. */
export const corner = (element: Element) => {
  const rect = px(element);
  const lean = {
    x: rect.x + rect.w / 2 < W / 2 ? 1 : -1,
    y: rect.y + rect.h / 2 < H / 2 ? 1 : -1,
  };
  return {
    at: {
      x: lean.x > 0 ? rect.x + rect.w : rect.x,
      y: lean.y > 0 ? rect.y + rect.h : rect.y,
    },
    lean,
  };
};

/** Where a mark's choices open: above it. */
export const anchorOf = (mark: Mark): Point => {
  if (mark.pen === "arrow" && mark.from && mark.to) {
    return {
      x: (mark.from.x + mark.to.x) / 2,
      y: Math.min(mark.from.y, mark.to.y),
    };
  }
  if (mark.pen === "counter") return corner(mark.element).at;
  const rect = px(mark.element);
  return { x: rect.x + rect.w / 2, y: rect.y };
};

/** The swatch a mark draws with: its own, or its look's default. */
export const swatchOf = (mark: Mark, lookId: LookId): Swatch =>
  mark.swatch ??
  ((mark.lookOverride ?? lookId) === "product" ? "ink" : "accent");
