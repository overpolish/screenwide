// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
import { headChoice } from "../../annotations/annotation-choices";

import { ArrowHeadGlyph } from "./arrow-head-glyph";
import { PenWeights } from "./pen-weights";

import type { SectionProps } from "./section-props";
import type { AnnotationHead } from "../../../bindings/AnnotationHead";

/** The arrow's pen: its weights and its heads, drawn in the text's colour as
 * the tool's icon is; the colour in hand shows in the row already, and a
 * dark one would hide them. */
export function ArrowPen({ onChange, ref, settings }: SectionProps) {
  return (
    <IslandCrossfade ref={ref}>
      <div className="flex gap-separate">
        <PenWeights onChange={onChange} settings={settings} />
        <PillGroup
          aria-label={headChoice.label()}
          items={headChoice.items().map((item) => ({
            ariaLabel: item.label ?? item.ariaLabel,
            icon: (
              <ArrowHeadGlyph
                className="size-icon-lg"
                head={item.id as AnnotationHead}
              />
            ),
            id: item.id,
          }))}
          onSelectionChange={(id) => {
            onChange({ defaultHead: headChoice.pick(id) });
          }}
          selected={headChoice.select(settings.defaultHead)}
        />
      </div>
    </IslandCrossfade>
  );
}
