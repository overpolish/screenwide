// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { LucideIcon } from "lucide-react";

import {
  ArrowToolIcon,
  CounterToolIcon,
  DrawToolIcon,
  FocusToolIcon,
  HighlightToolIcon,
  MagnifyToolIcon,
  RedactToolIcon,
  ShapeToolIcon,
  SpotlightToolIcon,
  ImageToolIcon,
  TextToolIcon,
} from "../../annotations/annotation-tool-icons";

import type { AnnotationKind } from "../../annotations/types";

/** Every tool the editor toolbar offers, under one name each. A drawing
 * tool's name is its [`AnnotationKind`], so the tool and the shape it makes
 * are one word. */
export type EditorToolId =
  | AnnotationKind
  | "captions"
  | "crop"
  | "cursor"
  | "frame"
  | "keyboard"
  | "scene"
  | "select"
  | "transcript";

type ViewTool = {
  drawsOnLayer?: false;
  isShifted?: never;
  /** The unmodified letter that takes the tool up. */
  shortcut?: string;
};

/** A tool that draws on a layer: it takes the newest layer in hand when
 * nothing is selected, and both editor toolbars build their toggle for it
 * out of this row, with the wording from `annotation-text`. */
type DrawingTool = {
  drawsOnLayer: true;
  icon: LucideIcon;
  shortcut: string;
  /** The letter is taken with Shift: the tool is another pen's second look
   * rather than a pen of its own, so it has no button or letter to itself. */
  isShifted?: true;
};

/**
 * The letter that takes each tool up, and - for the tools that draw - what
 * its toggle looks like.
 *
 * One table rather than a condition at each toolbar button, so a tool added
 * later states its behaviour in the same place as the rest. The glyph and the
 * wording are the live overlay's own for the same tool, so an annotation
 * reads the same whether it is drawn on a screenshot, on a recording or on
 * the desktop. A drawing tool's letter is the overlay's too
 * (`TOOL_KEYS` in `src-tauri/src/annotate/input/keys.rs`): the first letter
 * of its name where that is free, else the likeliest guess in it, the tool
 * used more often keeping a letter two want. None may be a mode's
 * (`editor-modes.ts`) or one of `editorToolKeyAction`'s own.
 */
const EDITOR_TOOLS: Record<AnnotationKind, DrawingTool> &
  Record<Exclude<EditorToolId, AnnotationKind>, ViewTool> = {
  arrow: {
    drawsOnLayer: true,
    icon: ArrowToolIcon,
    shortcut: "A",
  },
  captions: {},
  counter: {
    drawsOnLayer: true,
    icon: CounterToolIcon,
    shortcut: "C",
  },
  crop: { shortcut: "K" },
  cursor: {},
  draw: {
    drawsOnLayer: true,
    icon: DrawToolIcon,
    shortcut: "D",
  },
  focus: {
    drawsOnLayer: true,
    icon: FocusToolIcon,
    shortcut: "O",
  },
  frame: { shortcut: "F" },
  highlight: {
    drawsOnLayer: true,
    icon: HighlightToolIcon,
    shortcut: "H",
  },
  image: {
    drawsOnLayer: true,
    icon: ImageToolIcon,
    shortcut: "I",
  },
  keyboard: {},
  magnify: {
    drawsOnLayer: true,
    icon: MagnifyToolIcon,
    shortcut: "M",
  },
  redact: {
    drawsOnLayer: true,
    icon: RedactToolIcon,
    shortcut: "R",
  },
  scene: { shortcut: "L" },
  select: { shortcut: "V" },
  shape: {
    drawsOnLayer: true,
    icon: ShapeToolIcon,
    shortcut: "S",
  },
  spotlight: {
    drawsOnLayer: true,
    icon: SpotlightToolIcon,
    isShifted: true,
    shortcut: "O",
  },
  text: {
    drawsOnLayer: true,
    icon: TextToolIcon,
    shortcut: "T",
  },
  transcript: {},
};

const editorToolIds = () => Object.keys(EDITOR_TOOLS) as EditorToolId[];

/** Whether the tool draws annotations on a layer, which is also what makes
 * its id an [`AnnotationKind`]. */
export const isDrawingTool = (id: EditorToolId): id is AnnotationKind =>
  EDITOR_TOOLS[id].drawsOnLayer === true;

/** The drawing tools in the order the toolbars show them, most used first:
 * a narrow window moves the tail into its overflow menu, so the tools that
 * end up there are the ones least missed. */
const ANNOTATION_TOOL_ORDER: readonly AnnotationKind[] = [
  "image",
  "arrow",
  "text",
  "redact",
  "draw",
  "shape",
  "highlight",
  "counter",
  "spotlight",
  "focus",
  "magnify",
];

export const ANNOTATION_TOOLS = ANNOTATION_TOOL_ORDER.map((id) => ({
  ...EDITOR_TOOLS[id],
  id,
}));

export type AnnotationTool = (typeof ANNOTATION_TOOLS)[number];

/** Which shape the tool in hand draws, or null where it draws nothing - the
 * select tool, the crop, or no tool at all. The twin of `drawing_kind` in
 * `src-tauri/crates/annotations/src/gesture/mode.rs`. */
export const drawingToolKind = (tool: string | null | undefined) =>
  ANNOTATION_TOOLS.find((item) => item.id === tool)?.id ?? null;

/** Whether the annotation chrome answers to `tool`: a drawing tool, or the
 * select tool, which hit-tests the annotations without making any and draws
 * a band that chooses them. */
export const isAnnotationTool = (
  tool: string | null | undefined,
): tool is AnnotationKind | "select" =>
  tool === "select" || drawingToolKind(tool) !== null;

/** The letter a tool's button shows in its tooltip. */
export const toolShortcut = (id: EditorToolId) => EDITOR_TOOLS[id].shortcut;

/** The key that takes up a drawing tool, as a key event names it, for a
 * window that hands the press on to the editor. */
export const drawingToolKey = (id: AnnotationKind) => ({
  code: `Key${EDITOR_TOOLS[id].shortcut}`,
  modifiers: EDITOR_TOOLS[id].isShifted ? { shiftKey: true } : {},
});

/** Each tool that has a letter of its own, the letter, and whether it takes
 * Shift. */
export const EDITOR_TOOL_SHORTCUTS = editorToolIds().flatMap((id) => {
  const { isShifted = false, shortcut } = EDITOR_TOOLS[id];
  return shortcut === undefined ? [] : [{ id, isShifted, shortcut }];
});

/** Which tool a letter takes up, with Shift held when `isShifted`, or null
 * where the letter is not a tool's. The twin half of `editorToolKeyAction`,
 * which owns the keys that belong to no tool. */
export const editorToolForShortcut = (shortcut: string, isShifted = false) =>
  EDITOR_TOOL_SHORTCUTS.find(
    (tool) => tool.shortcut === shortcut && tool.isShifted === isShifted,
  )?.id ?? null;
