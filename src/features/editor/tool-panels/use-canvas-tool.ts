// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { usePopupPanelStore } from "../../popup-panel/store";
import { EditorKind } from "../types";

/** Panel activation retires the canvas tool, including its native interaction. */
export function useCanvasTool<T extends string>(
  workspace: EditorKind,
  initialTool: T,
) {
  const [tool, setTool] = useState<T | null>(() => {
    const content = usePopupPanelStore.getState().active?.content;
    return content?.kind === "tool" && content.workspace === workspace
      ? null
      : initialTool;
  });
  useEffect(() => {
    const synchronize = () => {
      const content = usePopupPanelStore.getState().active?.content;
      if (content?.kind === "tool" && content.workspace === workspace)
        setTool(null);
    };
    return usePopupPanelStore.subscribe(synchronize);
  }, [workspace]);
  return [tool, setTool] as const;
}
