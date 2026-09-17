// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import {
  ArrowToolIcon,
  CounterToolIcon,
} from "../../components/shared/annotation-style/mark-tool-icons";
import { AnnotateShape } from "../settings/types";

/** One tool the overlay can draw with. A shape added to the overlay becomes a
 * row here, and the toolbar grows a toggle without being touched.
 *
 * The glyph and the wording are the editor's own for the same tool, so a mark
 * reads the same whether it is drawn on a screenshot or on the desktop. The
 * twin of the arrow and counter tools in
 * `features/editor/components/screenshot-tools`. */
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
    icon: <ArrowToolIcon />,
    id: "arrow",
    label: "Arrow",
    name: "Draw an arrow",
    shortcut: "A",
  },
  {
    icon: <CounterToolIcon />,
    id: "counter",
    label: "Counter",
    name: "Drop a counter",
    shortcut: "N",
  },
];
