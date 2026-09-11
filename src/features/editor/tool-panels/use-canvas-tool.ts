// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import {
  activePopupPanel,
  PopupPanelContent,
  usePopupPanelStore,
} from "../../popup-panel/store";
import { EditorKind } from "../types";

import { toolPanelLabel } from "./tool-panel-window";

/**
 * Panel activation retires the canvas tool, including its native interaction.
 * The selection panel is the one that does not: it is the Select tool's own
 * controls, and the drag it describes has to stay live underneath it.
 */
const retiresCanvasTool = (
  content: PopupPanelContent | undefined,
  workspace: EditorKind,
) =>
  content?.kind === "tool" &&
  content.workspace === workspace &&
  content.tool !== "selection";

export function useCanvasTool<T extends string>(
  workspace: EditorKind,
  initialTool: T,
) {
  const panel = toolPanelLabel(workspace);
  const [tool, setTool] = useState<T | null>(() =>
    retiresCanvasTool(
      activePopupPanel(usePopupPanelStore.getState(), panel)?.content,
      workspace,
    )
      ? null
      : initialTool,
  );
  useEffect(() => {
    const synchronize = () => {
      const content = activePopupPanel(
        usePopupPanelStore.getState(),
        panel,
      )?.content;
      if (retiresCanvasTool(content, workspace)) setTool(null);
    };
    return usePopupPanelStore.subscribe(synchronize);
  }, [panel, workspace]);
  return [tool, setTool] as const;
}
