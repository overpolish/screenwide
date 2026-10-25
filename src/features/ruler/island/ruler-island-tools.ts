// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  type LucideIcon,
  RulerDimensionLine,
  SeparatorHorizontal,
  SeparatorVertical,
  SquareRoundCorner,
} from "lucide-react";

import { t } from "../../../i18n/i18n";

import type { RulerTolerance } from "../../../bindings/RulerTolerance";
import type { RulerTool } from "../../../bindings/RulerTool";

/** What hangs under the row: the edge tolerances, or how the tool in hand
 * is used. */
export type RulerSection = "edges" | "guide" | "measure" | "radius";

/** The shortcuts that stamp a measurement where the pointer is, as the
 * Ruler's settings bind them; `null` where one is unbound. */
export type RulerStampKeys = {
  horizontal: string | null;
  vertical: string | null;
};

export const RULER_TOOLS: readonly {
  icon: LucideIcon;
  id: RulerTool;
  name: () => string;
}[] = [
  {
    icon: RulerDimensionLine,
    id: "measure",
    name: () => t("ruler-island-measure"),
  },
  {
    icon: SeparatorVertical,
    id: "verticalGuide",
    name: () => t("ruler-island-vertical-guide"),
  },
  {
    icon: SeparatorHorizontal,
    id: "horizontalGuide",
    name: () => t("ruler-island-horizontal-guide"),
  },
  {
    icon: SquareRoundCorner,
    id: "radius",
    name: () => t("ruler-island-radius"),
  },
];

/** The edge tolerances, from the strongest edges only to the faintest, in
 * the order their key steps through them. */
export const RULER_TOLERANCES: readonly {
  id: RulerTolerance;
  name: () => string;
}[] = [
  { id: "clearEdges", name: () => t("overlay-ruler-clear-edges") },
  { id: "balanced", name: () => t("overlay-ruler-balanced") },
  { id: "subtleEdges", name: () => t("overlay-ruler-subtle-edges") },
];

/** The section the tool in hand hangs, if any: measuring names its stamp
 * keys, while both are bound. */
export function toolSection(
  tool: RulerTool,
  stampKeys: RulerStampKeys | null,
): RulerSection | null {
  if (tool === "radius") return "radius";
  if (tool !== "measure") return "guide";
  return stampKeys?.horizontal && stampKeys.vertical ? "measure" : null;
}
