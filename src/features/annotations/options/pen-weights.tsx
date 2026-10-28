// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
import { annotationSizeLabel } from "../annotation-text";
import { ANNOTATION_WIDTHS } from "../widths";

import { SizeDot } from "./size-dot";

/** The pen's weights as dots, each as wide as its stroke in proportion, the
 * thinnest at the smallest dot that reads. One pen draws the arrow, the
 * shape and a freehand stroke, so a weight looks the same for each. */
export function PenWeights({
  onChange,
  value,
}: {
  onChange: (points: number) => void;
  /** The stroke's width in points. */
  value: number;
}) {
  return (
    <PillGroup
      aria-label={annotationSizeLabel("width")}
      items={ANNOTATION_WIDTHS.map((points) => ({
        ariaLabel: t("annotate-island-points", { points }),
        icon: <SizeDot diameter={points / 2 + 2} />,
        id: String(points),
      }))}
      onSelectionChange={(id) => {
        onChange(Number(id));
      }}
      selected={String(value)}
    />
  );
}
