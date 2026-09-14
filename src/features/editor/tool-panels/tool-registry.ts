// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ToolPanelKind } from "../../popup-panel/store";

/** Every tool the editor toolbar offers, under one name each. */
export type EditorToolId =
  "arrow" | "crop" | "cursor" | "frame" | "keyboard" | "select";

type EditorTool = {
  /** The panel this tool opens, for the tools that have one. */
  panel?: ToolPanelKind;
  /** Opt in to a full-frame fit on selection and one-time window growth
   * when opening a panel. Omitted tools overlay without changing the view. */
  resetsView?: boolean;
};

/**
 * What the preview does when a tool is chosen.
 *
 * One table rather than a condition at each toolbar button, so a tool added
 * later states its behaviour in the same place as the rest.
 */
const EDITOR_TOOLS: Record<EditorToolId, EditorTool> = {
  // No panel yet: the arrow draws with the style it is given, and its
  // controls are a later slice.
  arrow: { resetsView: true },
  crop: { panel: "crop", resetsView: true },
  cursor: { panel: "cursor", resetsView: true },
  frame: { panel: "frame", resetsView: true },
  keyboard: { panel: "keyboard", resetsView: true },
  select: { panel: "selection", resetsView: false },
};

export const toolResetsView = (id: EditorToolId) =>
  EDITOR_TOOLS[id].resetsView === true;

export const toolPanel = (id: EditorToolId) => EDITOR_TOOLS[id].panel;

/** The same view policy, asked of an open panel rather than of a tool. */
export const panelResetsView = (panel: ToolPanelKind) =>
  (Object.keys(EDITOR_TOOLS) as EditorToolId[]).some(
    (id) => EDITOR_TOOLS[id].panel === panel && toolResetsView(id),
  );
