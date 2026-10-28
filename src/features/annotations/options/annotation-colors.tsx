// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ColorGrid } from "../../../composites/color-grid/color-grid";
import { ScrollArea } from "../../../primitives/scroll-area/scroll-area";

/** The colours an annotation is drawn in: the same grid of swatches, ten to
 * a row, wherever it is offered. Past five rows they scroll, so a long list
 * of colours of your own never pushes an island far down the screen. The
 * scrolled box holds the grid in the small inset, so a chosen swatch's
 * outline at its edge is not cut off; above and below it reaches past the
 * grid by as much, so the rows keep the section's own inset. Not at the
 * sides: an island takes the box's width as the section's, and a box
 * narrower than its grid is cut off. Where `custom` is given, a colour of
 * the user's own can be picked and kept; the live overlay's island leaves
 * it out, as its panel closes on every pick. */
export function AnnotationColors({
  custom,
  onChange,
  savedColors,
  value,
}: {
  onChange: (color: string) => void;
  value: string;
  /** Keeping a colour picked of the user's own, and forgetting one. */
  custom?: {
    onRemove: (color: string) => void;
    onSave: (color: string) => void;
  };
  savedColors?: string[];
}) {
  return (
    <ScrollArea
      className="p-inset-sm"
      constrainHeight
      rootClassName="-my-inset-sm max-h-34"
    >
      <ColorGrid
        columns={10}
        offersCustom={custom !== undefined}
        onChange={onChange}
        onRemoveColor={custom?.onRemove}
        onSaveColor={custom?.onSave}
        savedColors={savedColors}
        value={value}
      />
    </ScrollArea>
  );
}
