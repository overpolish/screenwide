// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { useSize } from "./island-size";

/** The island's parts as laid out, which its black is shaped round: the
 * elements holding the row, the line under it, the section and a column
 * opened beside the section, each told here as it mounts, and what each
 * needs. `fillsWidth` is whether the row is stretched across the island. */
export function useIslandMeasures(fillsWidth: boolean) {
  const [rowElement, setRowElement] = useState<HTMLDivElement | null>(null);
  const [sectionElement, setSectionElement] = useState<HTMLDivElement | null>(
    null,
  );
  const [belowElement, setBelowElement] = useState<HTMLDivElement | null>(null);
  const [aside, setAside] = useState<HTMLDivElement | null>(null);
  // What the row needs, not the island's width it may be stretched to.
  const rowSize = useSize(rowElement, { isNatural: fillsWidth });
  // What the section needs, not the box's width it may be stretched to.
  const sectionSize = useSize(sectionElement, { isNatural: true });
  const belowSize = useSize(belowElement);
  // Its last size outlives it, so a section without one reaches nowhere.
  const asideSize = useSize(aside);
  return {
    belowSize,
    reach: aside ? (asideSize?.width ?? 0) : 0,
    rowElement,
    rowSize,
    sectionSize,
    setAside,
    setBelowElement,
    setRowElement,
    setSectionElement,
  };
}
