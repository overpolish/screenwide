// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  DEFAULT_TOOL_PANEL_SNAPSHOT,
  ToolPanelSnapshot,
  useToolPanelStore,
} from "./tool-panel-store";

/** The panel window reads what the editor published, so a story seeds the
 * mirror the same way a live editor fills it. Both workspaces get the same
 * snapshot, so a story's workspace control shows what that workspace offers
 * rather than an empty panel. */
export const seedToolPanel = (snapshot: Partial<ToolPanelSnapshot>) => {
  const seeded = { ...DEFAULT_TOOL_PANEL_SNAPSHOT, ...snapshot };
  useToolPanelStore.setState({
    snapshots: { recording: seeded, screenshot: seeded },
  });
};
