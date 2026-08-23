// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

declare const rectSpace: unique symbol;

type Rect<Space extends "canvas" | "source"> = {
  height: number;
  readonly [rectSpace]: Space;
  width: number;
  x: number;
  y: number;
};

export type CanvasRect = Rect<"canvas">;
export type SourceRect = Rect<"source">;

export type CanvasGeometry = {
  /** The outer visible layer frame, including any Recenter inset. */
  frame: CanvasRect;
  /** The uniform source-image transform in canvas document coordinates. */
  sourceToCanvas: {
    scale: number;
    x: number;
    y: number;
  };
};

export type ScreenshotLayerGeometry = {
  canvas: CanvasGeometry;
  sourceCrop: SourceRect;
};

const clamp = (value: number, minimum: number, maximum: number) =>
  Math.min(maximum, Math.max(minimum, value));

const assertFiniteRect = (value: {
  height: number;
  width: number;
  x: number;
  y: number;
}) => {
  if (Object.values(value).some((coordinate) => !Number.isFinite(coordinate)))
    throw new Error("Rectangle coordinates must be finite");
};

export const canvasRect = (
  value: Omit<CanvasRect, typeof rectSpace>,
): CanvasRect => {
  assertFiniteRect(value);
  if (value.width <= 0 || value.height <= 0)
    throw new Error("Canvas rectangles must have positive dimensions");
  return { ...value } as CanvasRect;
};

export const sourceRect = (
  value: Omit<SourceRect, typeof rectSpace>,
): SourceRect => {
  assertFiniteRect(value);
  const firstX = clamp(Math.min(value.x, value.x + value.width), 0, 1);
  const firstY = clamp(Math.min(value.y, value.y + value.height), 0, 1);
  const secondX = clamp(Math.max(value.x, value.x + value.width), 0, 1);
  const secondY = clamp(Math.max(value.y, value.y + value.height), 0, 1);
  if (secondX <= firstX || secondY <= firstY)
    throw new Error("Source rectangles must overlap the source image");
  return {
    height: secondY - firstY,
    width: secondX - firstX,
    x: firstX,
    y: firstY,
  } as SourceRect;
};

export const fullSourceRect = () =>
  sourceRect({ height: 1, width: 1, x: 0, y: 0 });

export const translateSourceRect = (
  rect: SourceRect,
  delta: { x: number; y: number },
  bounds: SourceRect = fullSourceRect(),
): SourceRect => {
  const width = Math.min(rect.width, bounds.width);
  const height = Math.min(rect.height, bounds.height);
  return sourceRect({
    height,
    width,
    x:
      width === bounds.width
        ? bounds.x
        : clamp(rect.x + delta.x, bounds.x, bounds.x + bounds.width - width),
    y:
      height === bounds.height
        ? bounds.y
        : clamp(rect.y + delta.y, bounds.y, bounds.y + bounds.height - height),
  });
};

export const translateCanvasGeometry = (
  geometry: CanvasGeometry,
  delta: { x: number; y: number },
): CanvasGeometry => ({
  frame: canvasRect({
    ...geometry.frame,
    x: geometry.frame.x + delta.x,
    y: geometry.frame.y + delta.y,
  }),
  sourceToCanvas: {
    ...geometry.sourceToCanvas,
    x: geometry.sourceToCanvas.x + delta.x,
    y: geometry.sourceToCanvas.y + delta.y,
  },
});

export const scaleCanvasGeometry = (
  geometry: CanvasGeometry,
  anchor: { x: number; y: number },
  scale: number,
): CanvasGeometry => {
  const factor = Math.max(0.01, scale);
  const transform = (value: number, pivot: number) =>
    pivot + (value - pivot) * factor;
  return {
    frame: canvasRect({
      height: geometry.frame.height * factor,
      width: geometry.frame.width * factor,
      x: transform(geometry.frame.x, anchor.x),
      y: transform(geometry.frame.y, anchor.y),
    }),
    sourceToCanvas: {
      scale: geometry.sourceToCanvas.scale * factor,
      x: transform(geometry.sourceToCanvas.x, anchor.x),
      y: transform(geometry.sourceToCanvas.y, anchor.y),
    },
  };
};
