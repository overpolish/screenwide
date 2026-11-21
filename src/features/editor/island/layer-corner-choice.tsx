// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
import { CornerGlyph } from "../../annotations/options/shape-glyphs";
import {
  canvasShare,
  CIRCLE_PERCENT,
  CORNER_LOOKS,
  isSameMeasure,
  Measure,
  percent,
} from "../layer-measure";

// A layer's corners as looks: the same three for a screen, a screenshot's
// layer and a camera, shares of the canvas so a look rounds each alike, and
// for a camera a circle past them. The glyphs stop short of a circle for
// Round, so the camera's four read in order.

const LOOKS = [
  {
    glyph: 0,
    id: "square",
    label: () => t("annotate-island-corners-square"),
    radius: canvasShare(CORNER_LOOKS[0]),
  },
  {
    glyph: 20,
    id: "rounded",
    label: () => t("annotate-island-corners-rounded"),
    radius: canvasShare(CORNER_LOOKS[1]),
  },
  {
    glyph: 35,
    id: "round",
    label: () => t("annotate-island-corners-round"),
    radius: canvasShare(CORNER_LOOKS[2]),
  },
] as const;

const CIRCLE = {
  glyph: 50,
  id: "circle",
  label: () => t("annotate-island-corners-circle"),
  radius: percent(CIRCLE_PERCENT),
} as const;

/** A layer's corners. A radius no look gives leaves none chosen until one
 * is picked. */
export function LayerCornerChoice({
  hasCircle = false,
  onChange,
  value,
}: {
  onChange: (radius: Measure) => void;
  value: Measure;
  /** Whether a circle is offered past the looks, as it is for a camera. */
  hasCircle?: boolean;
}) {
  const looks = hasCircle ? [...LOOKS, CIRCLE] : LOOKS;
  return (
    <PillGroup
      aria-label={t("annotation-radius")}
      items={looks.map(({ glyph, id, label }) => ({
        ariaLabel: label(),
        icon: <CornerGlyph radius={glyph} />,
        id,
      }))}
      onSelectionChange={(id) => {
        const look = looks.find((candidate) => candidate.id === id);
        if (look) onChange(look.radius);
      }}
      selected={
        looks.find((look) => isSameMeasure(look.radius, value))?.id ?? ""
      }
    />
  );
}
