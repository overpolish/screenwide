// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ComponentType } from "react";

import { Focus } from "lucide-react";

import { t } from "../../../i18n/i18n";
import {
  ArrowToolIcon,
  CounterToolIcon,
  DrawToolIcon,
  HighlightToolIcon,
  ShapeToolIcon,
} from "../../annotations/annotation-tool-icons";

import { ArrowPen } from "./arrow-pen";
import { CounterPen } from "./counter-pen";
import { DrawPen } from "./draw-pen";
import { FocusPen } from "./focus-pen";
import { HighlightPen } from "./highlight-pen";
import { ShapePen } from "./shape-pen";

import type { SectionProps } from "./section-props";
import type { AnnotationKind } from "../../annotations/types";

/** The tools the islands offer, in the toolbar's order, each with its glyph
 * and the pen that hangs under the tools while it is in hand.
 *
 * The focus is one tool for drawing the eye to a box, by blurring what lies
 * around it or dimming it (`FocusPen`): blurred it draws a focus, dimmed a
 * spotlight. It is named here for both looks, and draws in no colour, so the
 * colour is not offered while it is in hand. */
export const ISLAND_TOOLS: {
  icon: typeof ArrowToolIcon;
  id: AnnotationKind;
  pen: ComponentType<SectionProps>;
  hasNoColor?: boolean;
  name?: () => string;
}[] = [
  { icon: ArrowToolIcon, id: "arrow", pen: ArrowPen },
  { icon: CounterToolIcon, id: "counter", pen: CounterPen },
  { icon: HighlightToolIcon, id: "highlight", pen: HighlightPen },
  { icon: ShapeToolIcon, id: "shape", pen: ShapePen },
  {
    hasNoColor: true,
    icon: Focus,
    id: "focus",
    name: () => t("annotate-island-focus-name"),
    pen: FocusPen,
  },
  { icon: DrawToolIcon, id: "draw", pen: DrawPen },
];
