// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Film,
  LayoutPanelLeft,
  type LucideIcon,
  MousePointer2,
  PenLine,
  SquarePen,
} from "lucide-react";

import { t } from "../../../i18n/i18n";
import { isWindows } from "../../../lib/platform";
import { annotationToolLabel } from "../../annotations/annotation-text";
import {
  LIBRARY_SHORTCUT,
  MODE_SHORTCUTS,
  TRACKS_SHORTCUT,
} from "../../editor/toolbar/editor-modes";
import {
  ANNOTATION_TOOLS,
  EDITOR_TOOL_SHORTCUTS,
  type EditorToolId,
} from "../../editor/tools/tool-registry";

import type { ShortcutScope } from "./shortcut-conflicts";
import type { FixedShortcutEntry } from "./shortcut-entry";
import type { AnnotationKind } from "../../annotations/types";

// The keys built into the editor and the live Annotate overlay, which are
// shown in the pane but not set there. The tools' and the modes' keys come
// from the tables the editor answers them from; the rest are the keys
// `useEditorWindowShortcuts` and the overlay's `input/keys.rs` hold.

type Row = { label: string; value: string; alternates?: string[] };

/** A key as a tooltip may write it, `F` or `Shift+KeyC`, in the hotkey
 * format the map reads. */
const asHotkey = (shortcut: string) =>
  /^[A-Z]$/.test(shortcut) ? `Key${shortcut}` : shortcut;

/** The key that takes up a tool, where it has one. */
const toolKey = (id: EditorToolId) => {
  const key = EDITOR_TOOL_SHORTCUTS.find((tool) => tool.id === id);
  return key && `${key.isShifted ? "Shift+" : ""}Key${key.shortcut}`;
};

/** The overlay's tools, which take the editor's letters (`TOOL_KEYS` in
 * `src-tauri/src/annotate/input/keys.rs`). */
const OVERLAY_TOOLS: readonly AnnotationKind[] = [
  "arrow",
  "counter",
  "highlight",
  "shape",
  "focus",
  "draw",
];

const toolRows = (tools: readonly (readonly [EditorToolId, string])[]) =>
  tools.flatMap(([id, label]) => {
    const value = toolKey(id);
    return value ? [{ label, value }] : [];
  });

const group = ({
  icon,
  isOff = false,
  name,
  rows,
  scope,
}: {
  icon: LucideIcon;
  name: string;
  rows: readonly Row[];
  scope: ShortcutScope;
  isOff?: boolean;
}): FixedShortcutEntry[] =>
  rows.map(({ alternates = [], label, value }) => ({
    alternates,
    group: name,
    icon,
    id: `${scope} ${label}`,
    isFixed: true,
    isOff,
    kind: "combo",
    label,
    scope,
    value,
  }));

const editorTools = (): Row[] =>
  toolRows([
    ["select", t("editor-tool-select")],
    ["crop", t("editor-tool-crop")],
    ...ANNOTATION_TOOLS.map(({ id }) => [id, annotationToolLabel(id)] as const),
  ]);

const editorModes = (): Row[] => [
  { label: t("editor-tool-frame"), value: asHotkey(MODE_SHORTCUTS.frame) },
  { label: t("settings-shortcut-editor-screen"), value: MODE_SHORTCUTS.screen },
  { label: t("editor-island-camera"), value: MODE_SHORTCUTS.camera },
  { label: t("editor-island-scene"), value: asHotkey(MODE_SHORTCUTS.scene) },
  { label: t("editor-island-cursor"), value: MODE_SHORTCUTS.cursor },
  { label: t("editor-island-keyboard"), value: MODE_SHORTCUTS.keyboard },
  {
    label: t("editor-island-captions"),
    value: asHotkey(MODE_SHORTCUTS.captions),
  },
  { label: t("editor-toolbar-transcript"), value: MODE_SHORTCUTS.transcript },
  { label: t("editor-island-sound"), value: MODE_SHORTCUTS.sound },
  { label: t("editor-timeline-tracks"), value: TRACKS_SHORTCUT },
  { label: t("editor-timeline-library"), value: LIBRARY_SHORTCUT },
];

const ARROWS = ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"];

const editorEditing = (): Row[] => [
  {
    label: t("settings-shortcut-editor-undo"),
    value: "CommandOrControl+KeyZ",
  },
  {
    // Windows apps answer Ctrl+Y as well; a Mac's ⌘Y is not redo.
    alternates: isWindows() ? ["Control+KeyY"] : [],
    label: t("settings-shortcut-editor-redo"),
    value: "CommandOrControl+Shift+KeyZ",
  },
  {
    label: t("settings-shortcut-editor-copy"),
    value: "CommandOrControl+KeyC",
  },
  {
    label: t("settings-shortcut-editor-paste"),
    value: "CommandOrControl+KeyV",
  },
  {
    alternates: ["Delete"],
    label: t("settings-shortcut-editor-delete"),
    value: "Backspace",
  },
  { label: t("settings-shortcut-editor-back-to-select"), value: "Escape" },
  {
    label: t("settings-shortcut-editor-deselect-all"),
    value: "CommandOrControl+KeyD",
  },
  { label: t("settings-shortcut-editor-confirm-crop"), value: "Enter" },
  { label: t("editor-arrange-forward"), value: "BracketRight" },
  { label: t("editor-arrange-front"), value: "Shift+BracketRight" },
  { label: t("editor-arrange-backward"), value: "BracketLeft" },
  { label: t("editor-arrange-back"), value: "Shift+BracketLeft" },
  {
    alternates: ARROWS.slice(1),
    label: t("settings-shortcut-editor-nudge"),
    value: ARROWS[0],
  },
  {
    alternates: ARROWS.slice(1).map((arrow) => `Shift+${arrow}`),
    label: t("settings-shortcut-editor-nudge-further"),
    value: `Shift+${ARROWS[0]}`,
  },
  { label: t("editor-export"), value: "CommandOrControl+KeyE" },
];

const editorTimeline = (): Row[] => [
  {
    alternates: ["KeyP"],
    label: t("settings-shortcut-editor-play"),
    value: "Space",
  },
  { label: t("settings-shortcut-editor-step-back"), value: "ArrowLeft" },
  { label: t("settings-shortcut-editor-step-forward"), value: "ArrowRight" },
  {
    label: t("settings-shortcut-editor-jump-back"),
    value: "Shift+ArrowLeft",
  },
  {
    label: t("settings-shortcut-editor-jump-forward"),
    value: "Shift+ArrowRight",
  },
  { label: t("settings-shortcut-editor-start"), value: "Home" },
  { label: t("settings-shortcut-editor-end"), value: "End" },
  { label: t("editor-timeline-cut"), value: "KeyB" },
  {
    label: t("settings-shortcut-editor-split"),
    value: "CommandOrControl+KeyB",
  },
  { label: t("editor-timeline-range"), value: "Shift+KeyR" },
  { label: t("settings-shortcut-editor-snapping"), value: "KeyN" },
];

const annotateKeys = (): Row[] => [
  ...toolRows(
    OVERLAY_TOOLS.map((kind) => [kind, annotationToolLabel(kind)] as const),
  ),
  {
    label: t("settings-shortcut-annotate-undo"),
    value: "CommandOrControl+KeyZ",
  },
  {
    alternates: ["Delete"],
    label: t("settings-shortcut-annotate-clear"),
    value: "Backspace",
  },
  { label: t("settings-shortcut-annotate-leave"), value: "Escape" },
];

/** The editor's keys, grouped as its parts are, and the overlay's. */
export const fixedShortcutEntries = ({
  isAnnotateOff,
}: {
  isAnnotateOff: boolean;
}): FixedShortcutEntry[] => [
  ...group({
    icon: MousePointer2,
    name: t("settings-shortcuts-editor-tools"),
    rows: editorTools(),
    scope: "editor",
  }),
  ...group({
    icon: LayoutPanelLeft,
    name: t("settings-shortcuts-editor-modes"),
    rows: editorModes(),
    scope: "editor",
  }),
  ...group({
    icon: SquarePen,
    name: t("settings-shortcuts-editor-editing"),
    rows: editorEditing(),
    scope: "editor",
  }),
  ...group({
    icon: Film,
    name: t("settings-shortcuts-editor-timeline"),
    rows: editorTimeline(),
    scope: "editor",
  }),
  ...group({
    icon: PenLine,
    isOff: isAnnotateOff,
    name: t("settings-section-annotate"),
    rows: annotateKeys(),
    scope: "annotate",
  }),
];

/** The labels of the built-in keys, by group, for the settings search. */
export const fixedShortcutGroups = () => {
  const groups = new Map<string, string[]>();
  for (const entry of fixedShortcutEntries({ isAnnotateOff: false }))
    groups.set(entry.group, [...(groups.get(entry.group) ?? []), entry.label]);
  return groups;
};
