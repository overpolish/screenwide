// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import {
  CounterAngleDial,
  CounterSizeChoice,
} from "../../annotations/options/counter-choices";

import type { SectionProps } from "./section-props";

/** The counter's pen: its disc sizes and where its tail points. A counter
 * cannot be picked up again once dropped, so it is aimed before it is
 * drawn. */
export function CounterPen({ onChange, settings }: SectionProps) {
  return (
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
  );
}
