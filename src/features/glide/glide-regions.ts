// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { GlideRegion } from "../../bindings/GlideRegion";
export type { GlideRegion };

/** Rows in the fixed vertical grid every region is expressed on. */
export const glideGridRows = 2;

export const sameRegion = (a: GlideRegion | null, b: GlideRegion | null) =>
  a === b ||
  (a !== null &&
    b !== null &&
    a.gridCols === b.gridCols &&
    a.colStart === b.colStart &&
    a.colSpan === b.colSpan &&
    a.rowStart === b.rowStart &&
    a.rowSpan === b.rowSpan);
