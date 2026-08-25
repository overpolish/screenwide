// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ResizeDirection } from "./types";

type Size = { height: number; width: number };
type Point = { x: number; y: number };
type Rect = Point & Size;

/** The current region boundary point represented by the active resize handle. */
export const magnifierHandlePoint = (
  rect: Rect,
  direction: ResizeDirection | undefined,
  pointer: Point | null,
): Point => {
  const normalized = direction?.toLowerCase();
  let x = rect.x + rect.width / 2;
  let y = rect.y + rect.height / 2;

  if (normalized?.includes("left")) x = rect.x;
  if (normalized?.includes("right")) x = rect.x + rect.width;
  if (normalized?.includes("top")) y = rect.y;
  if (normalized?.includes("bottom")) y = rect.y + rect.height;
  if ((normalized === "top" || normalized === "bottom") && pointer) {
    x = pointer.x;
  }
  if ((normalized === "left" || normalized === "right") && pointer) {
    y = pointer.y;
  }

  return {
    x: Math.max(rect.x, Math.min(x, rect.x + rect.width)),
    y: Math.max(rect.y, Math.min(y, rect.y + rect.height)),
  };
};

/** Maps WebView client coordinates into the monitor capture's pixel space. */
export const magnifierCapturePoint = (
  point: Point,
  viewport: Size,
  capture: Size,
): Point => ({
  x: point.x * (capture.width / viewport.width),
  y: point.y * (capture.height / viewport.height),
});
