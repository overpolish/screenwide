// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import {
  CounterAngleDial,
  CounterSizeChoice,
} from "../../annotations/options/counter-choices";

import type { SectionProps } from "./section-props";

/** The counter's pen: its disc sizes and where its tail points. A counter
 * cannot be picked up again once dropped, so it is aimed before it is
 * drawn. */
export function CounterPen({ onChange, ref, settings }: SectionProps) {
  return (
    <IslandCrossfade ref={ref}>
      <div className="flex gap-separate">
        <CounterSizeChoice
          onChange={(defaultCounterSize) => {
            onChange({ defaultCounterSize });
          }}
          value={settings.defaultCounterSize}
        />
        <CounterAngleDial
          onChange={(defaultCounterAngle) => {
            onChange({ defaultCounterAngle });
          }}
          value={settings.defaultCounterAngle}
        />
      </div>
    </IslandCrossfade>
  );
}
