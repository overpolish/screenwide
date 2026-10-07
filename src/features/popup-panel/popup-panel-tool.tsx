// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { ToolPanel } from "../editor/tool-panels/tool-panel";

import { PopupPanelToolContent } from "./store";
import { useFittedPanel } from "./use-fitted-panel";

/**
 * The panel as an editor tool's own controls.
 *
 * Unlike a list it has no ceiling of its own: it is as tall as the controls
 * it holds, up to what the screen can show.
 */
export function PopupPanelTool({
  content,
}: {
  content: PopupPanelToolContent;
}) {
  const contentRef = useRef<HTMLDivElement>(null);
  useFittedPanel(contentRef, `${content.workspace}:${content.tool}`);

  return (
    <div className="w-full" ref={contentRef}>
      <ToolPanel tool={content.tool} workspace={content.workspace} />
    </div>
  );
}
