// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** A rectangle on the source image, in shares of its width and height. */
export type SourceRect = {
  height: number;
  width: number;
  x: number;
  y: number;
};

const clamp = (value: number, minimum: number, maximum: number) =>
  Math.min(maximum, Math.max(minimum, value));

const assertFiniteRect = (value: SourceRect) => {
  if (Object.values(value).some((coordinate) => !Number.isFinite(coordinate)))
    throw new Error("Rectangle coordinates must be finite");
};

export const sourceRect = (value: SourceRect): SourceRect => {
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
  };
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
