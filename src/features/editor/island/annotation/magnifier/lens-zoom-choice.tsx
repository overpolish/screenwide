// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../../i18n/i18n";
import { PillGroup } from "../../../../../primitives/pill-group/pill-group";
import { annotationSizeLabel } from "../../../../annotations/annotation-text";
import { LENS_ZOOMS } from "../../../annotations/annotation-shape-parsers";

/** A zoom as the lens seen face on, an outline in the text's colour at the
 * icons' stroke, with what it shows drawn larger the more it enlarges: a dot
 * the zoom's size, so the four looks read as one lens closing in. */
function ZoomGlyph({ zoom }: { zoom: number }) {
  return (
    <svg
      aria-hidden
      className="size-icon-lg"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      viewBox="0 0 24 24"
    >
      <circle cx={12} cy={12} r={8} />
      <circle
        cx={12}
        cy={12}
        fill="currentColor"
        r={1.25 * zoom}
        stroke="none"
      />
    </svg>
  );
}

/** How many times a lens enlarges what lies under its middle, from its four
 * looks. */
export function LensZoomChoice({
  onChange,
  value,
}: {
  onChange: (zoom: number) => void;
  /** How many times the lens enlarges. A zoom between the looks, kept from
   * a document, leaves none chosen until one is picked. */
  value: number;
}) {
  return (
    <PillGroup
      aria-label={annotationSizeLabel("zoom")}
      items={LENS_ZOOMS.map((zoom) => ({
        ariaLabel: t("annotate-island-zoom", { zoom }),
        icon: <ZoomGlyph zoom={zoom} />,
        id: String(zoom),
      }))}
      onSelectionChange={(id) => {
        onChange(Number(id));
      }}
      selected={String(value)}
    />
  );
}
