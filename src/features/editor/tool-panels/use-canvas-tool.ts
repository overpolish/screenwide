// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import {
  activePopupPanel,
  PopupPanelContent,
  ToolPanelKind,
  usePopupPanelStore,
} from "../../popup-panel/store";
import { EditorKind } from "../types";

import { toolPanelLabel } from "./tool-panel-window";

/**
 * Panel activation retires the canvas tool, including its native interaction.
 * The exceptions are the panels that are a canvas tool's own controls - Select,
 * Frame and Crop: the drag each of them describes has to stay live underneath
 * the panel that names it. The Arrow panel is the same case one step in: it
 * dresses the annotation the tool has in hand, so the tool holding it stays up.
 */
const canvasToolPanels: ToolPanelKind[] = [
  "annotation",
  "crop",
  "frame",
  "selection",
];

const retiresCanvasTool = (
  content: PopupPanelContent | undefined,
  workspace: EditorKind,
) =>
  content?.kind === "tool" &&
  content.workspace === workspace &&
  !canvasToolPanels.includes(content.tool);

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
