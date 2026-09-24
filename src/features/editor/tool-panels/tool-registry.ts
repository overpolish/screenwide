// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { LucideIcon } from "lucide-react";

import {
  ArrowToolIcon,
  CounterToolIcon,
  TextToolIcon,
} from "../../../components/shared/annotation-style/annotation-tool-icons";
import { ToolPanelKind } from "../../popup-panel/store";

import type { AnnotationKind } from "../../../components/shared/annotation-style/types";

/** Every tool the editor toolbar offers, under one name each. A drawing
 * tool's name is its [`AnnotationKind`], so the tool and the shape it makes
 * are one word. */
export type EditorToolId =
  AnnotationKind | "crop" | "cursor" | "frame" | "keyboard" | "select";

type ViewTool = {
  drawsOnLayer?: false;
  /** The panel this tool opens, for the tools that have one. */
  panel?: ToolPanelKind;
  /** Opt in to a full-frame fit on selection and one-time window growth
   * when opening a panel. Omitted tools overlay without changing the view. */
  resetsView?: boolean;
  /** The unmodified letter that takes the tool up. */
  shortcut?: string;
};

/** A tool that draws on a layer: it takes the newest layer in hand when
 * nothing is selected, and both editor toolbars build their toggle for it
 * out of this row. */
type DrawingTool = {
  drawsOnLayer: true;
  icon: LucideIcon;
  label: string;
  /** Accessible name, which says what the tool acts on. */
  name: string;
  shortcut: string;
  panel?: ToolPanelKind;
  resetsView?: boolean;
};

/**
 * What the preview does when a tool is chosen, and - for the tools that draw
 * - what its toggle looks like.
 *
 * One table rather than a condition at each toolbar button, so a tool added
 * later states its behaviour in the same place as the rest. The glyph and the
 * wording are the live overlay's own for the same tool, so an annotation
 * reads the same whether it is drawn on a screenshot, on a recording or on
 * the desktop.
 */
const EDITOR_TOOLS: Record<AnnotationKind, DrawingTool> &
  Record<Exclude<EditorToolId, AnnotationKind>, ViewTool> = {
  // The drawing tools own no panel: their controls belong to the annotation in
  // hand rather than to the tool, and follow the selection instead. They also
  // change no layout, so there is nothing for a fit to reclaim and the zoom a
  // drawing was aimed at survives being picked up.
  arrow: {
    drawsOnLayer: true,
    icon: ArrowToolIcon,
    label: "Arrow",
    name: "Draw an arrow",
    shortcut: "A",
  },
  counter: {
    drawsOnLayer: true,
    icon: CounterToolIcon,
    label: "Counter",
    name: "Drop a counter",
    shortcut: "N",
  },
  crop: { panel: "crop", resetsView: true, shortcut: "C" },
  cursor: { panel: "cursor", resetsView: true },
  frame: { panel: "frame", resetsView: true, shortcut: "F" },
  keyboard: { panel: "keyboard", resetsView: true },
  select: { panel: "selection", resetsView: false, shortcut: "V" },
  text: {
    drawsOnLayer: true,
    icon: TextToolIcon,
    label: "Text",
    name: "Add text",
    shortcut: "T",
  },
};

const editorToolIds = () => Object.keys(EDITOR_TOOLS) as EditorToolId[];

/** Whether the tool draws annotations on a layer, which is also what makes
 * its id an [`AnnotationKind`]. */
export const isDrawingTool = (id: EditorToolId): id is AnnotationKind =>
  EDITOR_TOOLS[id].drawsOnLayer === true;

/** The drawing tools, in the order the toolbars show them. */
export const ANNOTATION_TOOLS = editorToolIds().flatMap((id) =>
  isDrawingTool(id) ? [{ ...EDITOR_TOOLS[id], id }] : [],
);

/** Which shape the tool in hand draws, or null where it draws nothing - the
 * select tool, the crop, or no tool at all. The twin of `drawing_kind` in
 * `src-tauri/src/editor/annotations/gesture.rs`. */
export const drawingToolKind = (tool: string | null | undefined) =>
  ANNOTATION_TOOLS.find((item) => item.id === tool)?.id ?? null;

/** Whether the annotation chrome answers to `tool`: a drawing tool, or the
 * select tool, which hit-tests the annotations without making any. */
export const isAnnotationTool = (
  tool: string | null | undefined,
): tool is AnnotationKind | "select" =>
  tool === "select" || drawingToolKind(tool) !== null;

/** The letter that takes up a drawing tool. */
export const drawingToolShortcut = (id: AnnotationKind) =>
  EDITOR_TOOLS[id].shortcut;

/** Each tool that has a letter of its own, and the letter. */
export const EDITOR_TOOL_SHORTCUTS = editorToolIds().flatMap((id) => {
  const { shortcut } = EDITOR_TOOLS[id];
  return shortcut === undefined ? [] : [{ id, shortcut }];
});

/** Which tool a plain letter takes up, or null where the letter is not a
 * tool's. The twin half of `editorToolKeyAction`, which owns the keys that
 * belong to no tool. */
export const editorToolForShortcut = (shortcut: string) =>
  EDITOR_TOOL_SHORTCUTS.find((tool) => tool.shortcut === shortcut)?.id ?? null;

export const toolResetsView = (id: EditorToolId) =>
  EDITOR_TOOLS[id].resetsView === true;

export const toolPanel = (id: EditorToolId) => EDITOR_TOOLS[id].panel;

/** The same view policy, asked of an open panel rather than of a tool. */
export const panelResetsView = (panel: ToolPanelKind) =>
  editorToolIds().some(
    (id) => EDITOR_TOOLS[id].panel === panel && toolResetsView(id),
  );
