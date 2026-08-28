// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export type GridNavigationDirection = "down" | "left" | "right" | "up";

type GridNavigationPosition = {
  columns: number;
  currentIndex: number;
  direction: GridNavigationDirection;
  itemCount: number;
};

export function getNextGridItemIndex({
  columns,
  currentIndex,
  direction,
  itemCount,
}: GridNavigationPosition): number {
  if (
    currentIndex < 0 ||
    currentIndex >= itemCount ||
    itemCount <= 0 ||
    columns <= 0
  ) {
    return currentIndex;
  }

  const column = currentIndex % columns;

  switch (direction) {
    case "left":
      return column > 0 ? currentIndex - 1 : currentIndex;
    case "right":
      return column < columns - 1 && currentIndex + 1 < itemCount
        ? currentIndex + 1
        : currentIndex;
    case "up":
      return currentIndex - columns >= 0
        ? currentIndex - columns
        : currentIndex;
    case "down":
      return currentIndex + columns < itemCount
        ? currentIndex + columns
        : currentIndex;
  }
}
