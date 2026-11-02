// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type LucideIcon } from "lucide-react";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

// A Lucide glyph painted on a canvas, from the same shapes the icon draws
// in the page, so a painted control carries the glyph its real one does.

const shapes = new Map<LucideIcon, Element[]>();

/** The icon's shapes, read once from the icon's own markup. */
function shapesOf(icon: LucideIcon) {
  const known = shapes.get(icon);
  if (known) return known;
  const markup = renderToStaticMarkup(createElement(icon));
  const svg = new DOMParser().parseFromString(markup, "image/svg+xml");
  const read = [...svg.documentElement.children];
  shapes.set(icon, read);
  return read;
}

const number = (shape: Element, name: string) =>
  Number(shape.getAttribute(name) ?? 0);

/** The shape as a path, in the icon's 24-unit box. */
function pathOf(shape: Element) {
  const path = new Path2D();
  switch (shape.tagName) {
    case "path":
      path.addPath(new Path2D(shape.getAttribute("d") ?? ""));
      break;
    case "circle":
      path.arc(
        number(shape, "cx"),
        number(shape, "cy"),
        number(shape, "r"),
        0,
        Math.PI * 2,
      );
      break;
    case "rect":
      path.roundRect(
        number(shape, "x"),
        number(shape, "y"),
        number(shape, "width"),
        number(shape, "height"),
        number(shape, "rx"),
      );
      break;
    case "line":
      path.moveTo(number(shape, "x1"), number(shape, "y1"));
      path.lineTo(number(shape, "x2"), number(shape, "y2"));
      break;
    case "polyline":
    case "polygon": {
      const points = (shape.getAttribute("points") ?? "")
        .trim()
        .split(/[\s,]+/)
        .map(Number);
      for (let index = 0; index + 1 < points.length; index += 2) {
        if (index === 0) path.moveTo(points[0], points[1]);
        else path.lineTo(points[index], points[index + 1]);
      }
      if (shape.tagName === "polygon") path.closePath();
      break;
    }
  }
  return path;
}

/** `icon` stroked in `colour`, `size` points square, its corner at
 * (`x`, `y`). */
export function paintGlyph(
  context: CanvasRenderingContext2D,
  icon: LucideIcon,
  {
    colour,
    size,
    x,
    y,
  }: { colour: string; size: number; x: number; y: number },
) {
  context.save();
  context.translate(x, y);
  context.scale(size / 24, size / 24);
  context.strokeStyle = colour;
  context.lineWidth = 2;
  context.lineCap = "round";
  context.lineJoin = "round";
  for (const shape of shapesOf(icon)) context.stroke(pathOf(shape));
  context.restore();
}
