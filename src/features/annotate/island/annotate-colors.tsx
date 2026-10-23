// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import { ScrollArea } from "../../../primitives/scroll-area/scroll-area";
import { AnnotationColorGrid } from "../../annotations/annotation-color-grid";

import type { SectionProps } from "./section-props";

/** The colours every tool draws in, as the editor offers them: the same grid
 * of swatches, ten to a row. Past five rows they scroll, so a long list of
 * colours of your own never pushes the island far down the screen. The
 * scrolled box holds the grid in the small inset, so a chosen swatch's
 * outline at its edge is not cut off; above and below it reaches past the
 * grid by as much, so the rows keep the section's own inset. Not at the
 * sides: the island takes the box's width as the section's, and a box
 * narrower than its grid is cut off. */
export function AnnotateColors({
  onChange,
  ref,
  savedColors,
  settings,
}: SectionProps & { savedColors?: string[] }) {
  return (
    <IslandCrossfade ref={ref}>
      <ScrollArea
        className="p-inset-sm"
        constrainHeight
        rootClassName="-my-inset-sm max-h-34"
      >
        <AnnotationColorGrid
          columns={10}
          onChange={(defaultColor) => {
            onChange({ defaultColor });
          }}
          savedColors={savedColors}
          size="sm"
          value={settings.defaultColor}
        />
      </ScrollArea>
    </IslandCrossfade>
  );
}
