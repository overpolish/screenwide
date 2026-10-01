// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

const BASE_HEIGHT_FRACTION = 60 / 1080;
const DESIGN_HEIGHT = 20;
const EDGE_MARGIN_FRACTION = 0.055;
const ANIMATION_EXTENT = 1.12;

export const keyboardDefaultCenter = ({
  positionXPercent,
  positionYPercent,
  sizePercent,
}: {
  sizePercent: number;
  positionXPercent?: number;
  positionYPercent?: number;
}) => ({
  x: positionXPercent === undefined ? 0.5 : positionXPercent / 100,
  y:
    positionYPercent === undefined
      ? 1 -
        EDGE_MARGIN_FRACTION -
        (BASE_HEIGHT_FRACTION * (sizePercent / 100)) / 2
      : positionYPercent / 100,
});

export const keyboardMaximumSizePercent = ({
  height,
  maximumWidthUnits,
  width,
}: {
  height: number;
  width: number;
  maximumWidthUnits?: number | null;
}) => {
  if (
    maximumWidthUnits == null ||
    maximumWidthUnits <= 0 ||
    width <= 0 ||
    height <= 0
  )
    return 500;
  const availableWidth = width * (1 - EDGE_MARGIN_FRACTION * 2);
  const widthAtUnitScale =
    height * BASE_HEIGHT_FRACTION * (maximumWidthUnits / DESIGN_HEIGHT);
  const exact = (availableWidth / (widthAtUnitScale * ANIMATION_EXTENT)) * 100;
  return Math.max(5, Math.min(500, Math.floor(exact / 5) * 5));
};

export const keyboardSelectionGeometry = ({
  height,
  maximumWidthUnits,
  position,
  positionXPercent,
  positionYPercent,
  sizePercent,
  width,
}: {
  height: number;
  sizePercent: number;
  width: number;
  maximumWidthUnits?: number | null;
  position?: { centerX: number; centerY: number; sizePercent?: number } | null;
  positionXPercent?: number;
  positionYPercent?: number;
}) => {
  if (!maximumWidthUnits || width <= 0 || height <= 0) return null;
  const effectiveSizePercent = position?.sizePercent ?? sizePercent;
  const maximumSizePercent = keyboardMaximumSizePercent({
    height,
    maximumWidthUnits,
    width,
  });
  const fittedPercent = Math.min(effectiveSizePercent, maximumSizePercent);
  const selectionHeight = BASE_HEIGHT_FRACTION * (fittedPercent / 100);
  const selectionWidth = Math.min(
    1 - EDGE_MARGIN_FRACTION * 2,
    (height / width) * selectionHeight * (maximumWidthUnits / DESIGN_HEIGHT),
  );
  const defaultCenter = keyboardDefaultCenter({
    positionXPercent,
    positionYPercent,
    sizePercent: fittedPercent,
  });
  const centerX = position?.centerX ?? defaultCenter.x;
  const centerY = position?.centerY ?? defaultCenter.y;
  return {
    center: { x: centerX, y: centerY },
    defaultCenter,
    maximumSizePercent,
    minimumSizePercent: Math.min(50, maximumSizePercent),
    rect: {
      height: selectionHeight,
      width: selectionWidth,
      x: centerX - selectionWidth / 2,
      y: centerY - selectionHeight / 2,
    },
    sizePercent: fittedPercent,
  };
};
