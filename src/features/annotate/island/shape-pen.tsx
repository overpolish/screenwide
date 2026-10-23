// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { IslandChoice } from "../../../primitives/island/island-choice";
import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import { strokeChoice } from "../../annotations/annotation-choices";

import { CornerChoice } from "./corner-choice";
import { PenWeights } from "./pen-weights";
import { ShapeStrokeGlyph } from "./shape-glyphs";

import type { SectionProps } from "./section-props";

/** The shape's pen: its weight, its corners and whether it is drawn by hand,
 * in the order the tool panels keep: size, then geometry, then options. */
export function ShapePen({ onChange, ref, settings }: SectionProps) {
  return (
    <IslandCrossfade ref={ref}>
      <div className="flex gap-separate">
        <PenWeights onChange={onChange} settings={settings} />
        <CornerChoice
          onChange={(shapeRadius) => {
            onChange({ shapeRadius });
          }}
          value={settings.shapeRadius}
        />
        <IslandChoice
          items={strokeChoice.items().map((item) => ({
            content: (
              <ShapeStrokeGlyph isHandDrawn={strokeChoice.pick(item.id)} />
            ),
            id: item.id,
            label: item.label ?? item.ariaLabel,
          }))}
          label={strokeChoice.label()}
          onChange={(id) => {
            onChange({ shapeHandDrawn: strokeChoice.pick(id) });
          }}
          value={strokeChoice.select(settings.shapeHandDrawn)}
        />
      </div>
    </IslandCrossfade>
  );
}
