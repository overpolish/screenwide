// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { EditorKind } from "../types";

import {
  DEFAULT_TOOL_PANEL_SNAPSHOT,
  ToolPanelSnapshot,
  useToolPanelStore,
} from "./tool-panel-store";

/** The panel window reads what the editor published, so a story seeds the
 * mirror the same way a live editor fills it. A panel reads the workspace it
 * was opened for, so a screenshot story seeds that one instead. */
export const seedToolPanel = (
  snapshot: Partial<ToolPanelSnapshot>,
  workspace: EditorKind = "recording",
) => {
  useToolPanelStore.setState({
    snapshots: { [workspace]: { ...DEFAULT_TOOL_PANEL_SNAPSHOT, ...snapshot } },
  });
};
