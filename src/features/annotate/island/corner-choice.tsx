// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { IslandChoice } from "../../../primitives/island/island-choice";

import { CornerGlyph } from "./shape-glyphs";

/** The corners a box is offered, as a percentage of its shorter side:
 * square, rounded by a quarter, which reads as plainly rounded rather than
 * softened, and rounded all the way, which is a circle. Presets rather than
 * a scale: three looks to pick between read at a glance, where a percentage
 * has to be tried to be seen. */
const CORNERS = [
  { id: "0", label: () => t("annotate-island-corners-square"), radius: 0 },
  { id: "25", label: () => t("annotate-island-corners-rounded"), radius: 25 },
  { id: "50", label: () => t("annotate-island-corners-round"), radius: 50 },
];

/** How round a shape's or a focus's corners are. A radius set to another
 * value elsewhere leaves no preset chosen until one is picked. */
export function CornerChoice({
  onChange,
  value,
}: {
  onChange: (radius: number) => void;
  /** Percent of the box's shorter side. */
  value: number;
}) {
  return (
    <IslandChoice
      items={CORNERS.map(({ id, label, radius }) => ({
        content: <CornerGlyph radius={radius} />,
        id,
        label: label(),
      }))}
      label={t("annotation-radius")}
      onChange={(id) => {
        onChange(Number(id));
      }}
      value={String(value)}
    />
  );
}
