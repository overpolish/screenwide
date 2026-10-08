// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import {
  ArrowToolIcon,
  CounterToolIcon,
  DrawToolIcon,
  HighlightToolIcon,
  ShapeToolIcon,
  SpotlightToolIcon,
} from "../../components/shared/annotation-style/annotation-tool-icons";
import { AnnotationKind } from "../../components/shared/annotation-style/types";

/** One tool the overlay can draw with. A shape added to the overlay becomes a
 * row here, and the toolbar grows a toggle without being touched.
 *
 * The glyph is the editor's own for the same tool, and the wording comes from
 * `annotation-text`, so an annotation reads the same whether it is drawn on a
 * screenshot or on the desktop. The twin of the same tools in
 * `features/editor/tool-panels/tool-registry`. */
type AnnotateTool = {
  icon: ReactNode;
  id: AnnotationKind;
  /** The unmodified letter that picks the tool up while drawing. The twin of
   * `TOOL_KEYS` in `src-tauri/src/annotate/input.rs`. */
  shortcut: string;
};

export const ANNOTATE_TOOLS: AnnotateTool[] = [
  { icon: <ArrowToolIcon />, id: "arrow", shortcut: "A" },
  { icon: <CounterToolIcon />, id: "counter", shortcut: "N" },
  { icon: <HighlightToolIcon />, id: "highlight", shortcut: "H" },
  { icon: <ShapeToolIcon />, id: "shape", shortcut: "O" },
  { icon: <SpotlightToolIcon />, id: "spotlight", shortcut: "S" },
  { icon: <DrawToolIcon />, id: "draw", shortcut: "D" },
];
