// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { IslandChoice } from "../../../primitives/island/island-choice";
import { annotationSizeLabel } from "../../annotations/annotation-text";
import { ANNOTATION_WIDTHS } from "../../annotations/widths";

import { SizeDot } from "./size-dot";

import type { SectionProps } from "./section-props";

/** The pen's weights as dots, each as wide as its stroke in proportion, the
 * thinnest at the smallest dot that reads. One pen draws the arrow and the
 * shape, so a weight chosen for either is the other's too. */
export function PenWeights({ onChange, settings }: Omit<SectionProps, "ref">) {
  return (
    <IslandChoice
      items={ANNOTATION_WIDTHS.map((points) => ({
        content: <SizeDot diameter={points / 2 + 2} />,
        id: String(points),
        label: t("annotate-island-points", { points }),
      }))}
      label={annotationSizeLabel("width")}
      onChange={(id) => {
        onChange({ defaultWidth: Number(id) });
      }}
      value={String(settings.defaultWidth)}
    />
  );
}
