// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { AngleDial } from "../../../primitives/angle-dial/angle-dial";
import { IslandCrossfade } from "../../../primitives/island/island-crossfade";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
import {
  dialDegrees,
  radiansFromDial,
} from "../../annotations/annotation-angle";
import { annotationSizeLabel } from "../../annotations/annotation-text";
import { annotationSizes } from "../../annotations/widths";

import { CounterPinIcon } from "./counter-pin-icon";
import { SizeDot } from "./size-dot";

import type { SectionProps } from "./section-props";

/** The counter's pen: its disc sizes as dots, a quarter of each disc's
 * diameter, and where its tail points as a dial. A counter cannot be picked
 * up again once dropped, so it is aimed before it is drawn. */
export function CounterPen({ onChange, ref, settings }: SectionProps) {
  return (
    <IslandCrossfade ref={ref}>
      <div className="flex gap-separate">
        <PillGroup
          aria-label={annotationSizeLabel("size")}
          items={annotationSizes("counter").map((points) => ({
            ariaLabel: t("annotate-island-points", { points }),
            icon: <SizeDot diameter={points / 4} />,
            id: String(points),
          }))}
          onSelectionChange={(id) => {
            onChange({ defaultCounterSize: Number(id) });
          }}
          selected={String(settings.defaultCounterSize)}
        />
        <AngleDial
          aria-label={t("annotation-angle")}
          onChange={(degrees) => {
            onChange({ defaultCounterAngle: radiansFromDial(degrees) });
          }}
          value={dialDegrees(settings.defaultCounterAngle)}
        >
          <CounterPinIcon className="size-icon" />
        </AngleDial>
      </div>
    </IslandCrossfade>
  );
}
