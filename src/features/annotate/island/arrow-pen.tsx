// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { ArrowHeadChoice } from "../../annotations/options/arrow-head-choice";
import { PenWeights } from "../../annotations/options/pen-weights";

import type { SectionProps } from "./section-props";

/** The arrow's pen: its weights and its heads. */
export function ArrowPen({ onChange, settings }: SectionProps) {
  return (
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
  );
}
