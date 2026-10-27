// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
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
        <PillGroup
          aria-label={strokeChoice.label()}
          items={strokeChoice.items().map((item) => ({
            ariaLabel: item.label ?? item.ariaLabel,
            icon: <ShapeStrokeGlyph isHandDrawn={strokeChoice.pick(item.id)} />,
            id: item.id,
          }))}
          onSelectionChange={(id) => {
            onChange({ shapeHandDrawn: strokeChoice.pick(id) });
          }}
          selected={strokeChoice.select(settings.shapeHandDrawn)}
        />
      </div>
    </IslandCrossfade>
  );
}
