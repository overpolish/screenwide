// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import { CornerChoice } from "../../annotations/options/corner-choice";
import { HandDrawnToggle } from "../../annotations/options/hand-drawn-toggle";
import { PenWeights } from "../../annotations/options/pen-weights";

import type { SectionProps } from "./section-props";

/** The shape's pen: its weight, its corners and whether it is drawn by hand,
 * in the order the tool panels keep: size, then geometry, then options. */
export function ShapePen({ onChange, ref, settings }: SectionProps) {
  return (
    <IslandCrossfade ref={ref}>
      <div className="flex gap-separate">
        <PenWeights
          onChange={(defaultWidth) => {
            onChange({ defaultWidth });
          }}
          value={settings.defaultWidth}
        />
        <CornerChoice
          onChange={(shapeRadius) => {
            onChange({ shapeRadius });
          }}
          value={settings.shapeRadius}
        />
        <HandDrawnToggle
          isHandDrawn={settings.shapeHandDrawn}
          kind="shape"
          onChange={(shapeHandDrawn) => {
            onChange({ shapeHandDrawn });
          }}
        />
      </div>
    </IslandCrossfade>
  );
}
