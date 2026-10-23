// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { IslandChoice } from "../../../primitives/island/island-choice";
import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
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
        <IslandChoice
          items={headChoice.items().map((item) => ({
            content: (
              <ArrowHeadGlyph
                className="size-icon-lg"
                head={item.id as AnnotationHead}
              />
            ),
            id: item.id,
            label: item.label ?? item.ariaLabel,
          }))}
          label={headChoice.label()}
          onChange={(id) => {
            onChange({ defaultHead: headChoice.pick(id) });
          }}
          value={headChoice.select(settings.defaultHead)}
        />
      </div>
    </IslandCrossfade>
  );
}
