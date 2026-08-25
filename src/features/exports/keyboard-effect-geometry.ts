// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

const BASE_HEIGHT_FRACTION = 60 / 1080;
const DESIGN_HEIGHT = 20;
const EDGE_MARGIN_FRACTION = 0.055;
const ANIMATION_EXTENT = 1.12;

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
