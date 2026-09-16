// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ArrowUpRight } from "lucide-react";
import { ReactNode } from "react";

import { AnnotateShape } from "../settings/types";

/** One tool the overlay can draw with. The overlay draws arrows and nothing
 * else so far; a second shape becomes a second row here, and the toolbar
 * grows a toggle without being touched.
 *
 * The glyph and the wording are the editor's own for the same tool, so the
 * arrow reads the same whether it is drawn on a screenshot or on the desktop.
 * The twin of the arrow tool in `features/editor/components/screenshot-tools`. */
type AnnotateTool = {
  icon: ReactNode;
  id: AnnotateShape;
  label: string;
  /** Accessible name, which says what the tool acts on. */
  name: string;
  /** The unmodified letter that picks the tool up while drawing. The twin of
   * `TOOL_KEYS` in `src-tauri/src/annotate/input.rs`. */
  shortcut: string;
};

export const ANNOTATE_TOOLS: AnnotateTool[] = [
  {
    icon: <ArrowUpRight />,
    id: "arrow",
    label: "Arrow",
    name: "Draw an arrow",
    shortcut: "A",
  },
];
