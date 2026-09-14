// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  DEFAULT_TOOL_PANEL_SNAPSHOT,
  ToolPanelSnapshot,
  useToolPanelStore,
} from "./tool-panel-store";

/** The panel window reads what the editor published, so a story seeds the
 * mirror the same way a live editor fills it. */
export const seedToolPanel = (snapshot: Partial<ToolPanelSnapshot>) => {
  useToolPanelStore.setState({
    snapshots: { recording: { ...DEFAULT_TOOL_PANEL_SNAPSHOT, ...snapshot } },
  });
};
