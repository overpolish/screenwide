// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { ToolPanelKind } from "../../popup-panel/store";
import { EditorKind } from "../types";

import { CursorPanel } from "./cursor-panel";
import { FramePanel } from "./frame-panel";
import { SelectionPanel } from "./selection-panel";
import { usePanelShortcuts } from "./use-panel-shortcuts";

const toolPanels: Record<
  ToolPanelKind,
  (props: { workspace: EditorKind }) => ReactNode
> = {
  cursor: CursorPanel,
  frame: FramePanel,
  selection: SelectionPanel,
};

/**
 * One editor tool's controls, as they appear in the panel window.
 *
 * The chrome is the panel's own: the window surface and its inset. The panel
 * is never anything but the tool in hand, so it does not name it; every tool
 * is presented the same way, and a panel is only ever asked for its controls.
 */
export function ToolPanel({
  tool,
  workspace,
}: {
  tool: ToolPanelKind;
  workspace: EditorKind;
}) {
  usePanelShortcuts(workspace);
  const Panel = toolPanels[tool];

  return (
    <section className="window-surface flex w-full flex-col gap-section overflow-hidden rounded-window p-window-inset text-content-fg">
      <Panel workspace={workspace} />
    </section>
  );
}
