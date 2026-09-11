// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { Text } from "../../../components/base/text/text";
import { ToolPanelKind } from "../../popup-panel/store";
import { EditorKind } from "../types";

import { CursorPanel } from "./cursor-panel";
import { toolPanelTitles } from "./tool-panel-titles";
import { usePanelShortcuts } from "./use-panel-shortcuts";

const toolPanels: Record<
  ToolPanelKind,
  (props: { workspace: EditorKind }) => ReactNode
> = {
  cursor: CursorPanel,
};

/**
 * One editor tool's controls, as they appear in the panel window.
 *
 * The chrome is the panel's own: the window surface, its inset, and the tool's
 * name above the controls. Every tool is presented the same way, so a panel
 * is only ever asked for its controls.
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
      <Text variant="headline">{toolPanelTitles[tool]}</Text>
      <Panel workspace={workspace} />
    </section>
  );
}
