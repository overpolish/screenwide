// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import { PenWeights } from "../../annotations/options/pen-weights";

import { SectionHint } from "./section-hint";

import type { SectionProps } from "./section-props";

/** The freehand pen: only its weight, as the overlay draws a stroke in the
 * colour and weight in hand and nothing else. Beside it, how a stroke held
 * still at its end is smoothed into a clean version of itself, whether a
 * line, a curve, an arrow or a closed shape
 * (`annotations/src/freehand/recognise.rs`), which nothing else shows. */
export function DrawPen({ onChange, ref, settings }: SectionProps) {
  return (
    <IslandCrossfade ref={ref}>
      <div className="flex items-center gap-separate">
        <PenWeights
          onChange={(defaultWidth) => {
            onChange({ defaultWidth });
          }}
          value={settings.defaultWidth}
        />
        <SectionHint>{t("annotate-island-draw-hint")}</SectionHint>
      </div>
    </IslandCrossfade>
  );
}
