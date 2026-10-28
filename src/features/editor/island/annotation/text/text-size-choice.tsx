// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../../i18n/i18n";
import { PillGroup } from "../../../../../primitives/pill-group/pill-group";
import { annotationSizeLabel } from "../../../../annotations/annotation-text";
import { annotationSizes } from "../../../../annotations/widths";

/** A type size as a capital A, drawn larger for each step, as the weights
 * are dots growing. */
function SizeGlyph({ step }: { step: number }) {
  const side = 8 + step * 2;
  return (
    <svg
      aria-hidden
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2.5}
      style={{ height: side, width: side }}
      viewBox="0 0 24 24"
    >
      <path d="M5 21 12 3l7 18M8 14h8" />
    </svg>
  );
}

/** How large a text is set, of the type sizes a text is offered. */
export function TextSizeChoice({
  onChange,
  value,
}: {
  onChange: (points: number) => void;
  /** The type size in points. */
  value: number;
}) {
  return (
    <PillGroup
      aria-label={annotationSizeLabel("size")}
      items={annotationSizes("text").map((points, step) => ({
        ariaLabel: t("annotate-island-points", { points }),
        icon: <SizeGlyph step={step} />,
        id: String(points),
      }))}
      onSelectionChange={(id) => {
        onChange(Number(id));
      }}
      selected={String(value)}
    />
  );
}
