// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** Rows in the fixed vertical grid every region is expressed on. */
export const glideGridRows = 2;

/**
 * A rectangle of grid cells with 0-based starts, on a grid of `gridCols`
 * columns (2 = halves, 3 = thirds) by `glideGridRows` rows.
 */
export type GlideRegion = {
  colSpan: number;
  colStart: number;
  gridCols: 2 | 3;
  rowSpan: number;
  rowStart: number;
};

export const fullHeight = { rowSpan: glideGridRows, rowStart: 0 };

/** The full-width bottom row, where a pending minimize converts into a move. */
export const bottomRowRegion = (gridCols: 2 | 3): GlideRegion => ({
  colSpan: gridCols,
  colStart: 0,
  gridCols,
  rowSpan: 1,
  rowStart: 1,
});

export const sameRegion = (a: GlideRegion | null, b: GlideRegion | null) =>
  a === b ||
  (a !== null &&
    b !== null &&
    a.gridCols === b.gridCols &&
    a.colStart === b.colStart &&
    a.colSpan === b.colSpan &&
    a.rowStart === b.rowStart &&
    a.rowSpan === b.rowSpan);

const columnLabel = ({ colSpan, colStart, gridCols }: GlideRegion) => {
  if (colSpan === gridCols) return "full width";
  const side = colStart === 0 ? "left" : "right";
  if (gridCols === 2) return `${side} half`;
  if (colSpan === 2) return `${side} two thirds`;
  return colStart === 1 ? "middle third" : `${side} third`;
};

/** Human-readable region name, for aria labels and debug readouts. */
export const describeRegion = (region: GlideRegion) => {
  const columns = columnLabel(region);
  const wide = columns === "full width";
  if (region.rowSpan === glideGridRows) return wide ? "full screen" : columns;
  const rows = region.rowStart === 0 ? "top half" : "bottom half";
  return wide ? rows : `${columns}, ${rows}`;
};

/** Re-expresses a region on the other column grid, keeping its anchored edge. */
export const regridRegion = (
  region: GlideRegion,
  thirds: boolean,
): GlideRegion => {
  const { colSpan, colStart } = region;
  if (thirds) return { ...region, colSpan: colSpan === 2 ? 3 : 2, gridCols: 3 };
  // Full width stays full, and the middle third has no half to anchor to.
  if (colSpan === 3 || (colStart === 1 && colSpan === 1)) {
    return { ...region, colSpan: 2, colStart: 0, gridCols: 2 };
  }
  const half = colStart === 0 ? 0 : 1;
  return { ...region, colSpan: 1, colStart: half, gridCols: 2 };
};

/**
 * The thirds ladder, left to right: every horizontal step moves one rung, so
 * each step travels in the direction of the swipe and the middle third sits two
 * steps from either fold. Full width is deliberately off the ladder - swiping
 * up fills instead.
 */
const columnLadder: Pick<GlideRegion, "colSpan" | "colStart">[] = [
  { colSpan: 1, colStart: 0 },
  { colSpan: 2, colStart: 0 },
  { colSpan: 1, colStart: 1 },
  { colSpan: 2, colStart: 1 },
  { colSpan: 1, colStart: 2 },
];

/** Where full width, reachable only by re-gridding a fill, joins the ladder. */
const fullWidthEntry = { left: 1, right: 3 };

/**
 * One column step. Thirds walk the ladder above by ±1, clamped so a push past
 * either end holds; full width joins at the two thirds of the step's direction.
 * Full-height halves keep their direct side toggle, while a half with a row
 * active caterpillars through the full-width row instead.
 */
export const stepColumns = (region: GlideRegion, step: number): GlideRegion => {
  const { colSpan, colStart, gridCols } = region;
  const far = step > 0 ? gridCols - 1 : 0;
  if (gridCols === 2) {
    if (colSpan === 2) return { ...region, colSpan: 1, colStart: far };
    if (colStart === far) return region;
    if (region.rowSpan < glideGridRows) {
      // Growing to the full-width row first carries the rows to the far side.
      return { ...region, colSpan: 2, colStart: 0 };
    }
    return { ...region, colStart: far };
  }
  const rung = columnLadder.findIndex(
    (position) =>
      position.colSpan === colSpan && position.colStart === colStart,
  );
  const next =
    rung < 0
      ? fullWidthEntry[step > 0 ? "right" : "left"]
      : Math.min(Math.max(rung + step, 0), columnLadder.length - 1);
  return { ...region, ...columnLadder[next] };
};

/** A caterpillar on the two-row grid: grow to full height, then follow. */
export const stepRows = (region: GlideRegion, step: number): GlideRegion => {
  const far = step > 0 ? glideGridRows - 1 : 0;
  if (region.rowSpan === glideGridRows) {
    return { ...region, rowSpan: 1, rowStart: far };
  }
  return region.rowStart === far ? region : { ...region, ...fullHeight };
};
