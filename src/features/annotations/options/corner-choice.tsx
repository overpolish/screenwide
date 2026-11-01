// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { PillGroup } from "../../../primitives/pill-group/pill-group";

import { CornerGlyph } from "./shape-glyphs";

/** The corners a box is offered, as a percentage of its shorter side:
 * square, rounded by a quarter, which reads as plainly rounded rather than
 * softened, and rounded all the way, which is a circle. Presets rather than
 * a scale: three looks to pick between read at a glance, where a percentage
 * has to be tried to be seen. */
const ANNOTATION_CORNERS = [
  { glyph: 0, radius: 0 },
  { glyph: 25, radius: 25 },
  { glyph: 50, radius: 50 },
] as const;

/** The three looks' names, square to round. */
const LOOKS = [
  () => t("annotate-island-corners-square"),
  () => t("annotate-island-corners-rounded"),
  () => t("annotate-island-corners-round"),
];

/** A preset: its radius, and the radius its glyph is drawn with, so three
 * presets close together still read apart at a glyph's size. */
type Corner = { readonly glyph: number; readonly radius: number };

/** How round a box's corners are: the only way to set a radius, as nothing
 * on the picture drags one. A radius at another value - one a saved project
 * kept from when its corner could be dragged - leaves no preset chosen until
 * one is picked. `corners` are the three looks, square to round, for a box
 * that wants other radii than a shape's, such as a whole picture, which a
 * quarter would round too far. */
export function CornerChoice({
  corners = ANNOTATION_CORNERS,
  onChange,
  value,
}: {
  onChange: (radius: number) => void;
  /** Percent of the box's shorter side. */
  value: number;
  corners?: readonly [Corner, Corner, Corner];
}) {
  return (
    <PillGroup
      aria-label={t("annotation-radius")}
      items={corners.map(({ glyph, radius }, index) => ({
        ariaLabel: LOOKS[index](),
        icon: <CornerGlyph radius={glyph} />,
        id: String(radius),
      }))}
      onSelectionChange={(id) => {
        onChange(Number(id));
      }}
      selected={String(value)}
    />
  );
}
