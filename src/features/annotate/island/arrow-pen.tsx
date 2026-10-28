// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import { ArrowHeadChoice } from "../../annotations/options/arrow-head-choice";
import { PenWeights } from "../../annotations/options/pen-weights";

import type { SectionProps } from "./section-props";

/** The arrow's pen: its weights and its heads. */
export function ArrowPen({ onChange, ref, settings }: SectionProps) {
  return (
    <IslandCrossfade ref={ref}>
      <div className="flex gap-separate">
        <PenWeights
          onChange={(defaultWidth) => {
            onChange({ defaultWidth });
          }}
          value={settings.defaultWidth}
        />
        <ArrowHeadChoice
          onChange={(defaultHead) => {
            onChange({ defaultHead });
          }}
          value={settings.defaultHead}
        />
      </div>
    </IslandCrossfade>
  );
}
