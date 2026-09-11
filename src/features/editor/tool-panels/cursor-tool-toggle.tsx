// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { MousePointer } from "lucide-react";
import { useRef } from "react";

import { PreviewToolToggle } from "../components/preview-tool-toggle";

import { toolPanelTitles } from "./tool-panel-titles";
import { useToolPanel } from "./use-tool-panel";

/**
 * The editor toolbar's way into the cursor panel. A second press on it puts
 * the panel away, which is why the button's own bounds travel with it.
 */
export function CursorToolToggle() {
  const { openTool, toggle } = useToolPanel("recording");
  const triggerRef = useRef<HTMLSpanElement>(null);

  return (
    <span className="inline-flex" data-editor-tool="cursor" ref={triggerRef}>
      <PreviewToolToggle
        isSelected={openTool === "cursor"}
        label={toolPanelTitles.cursor}
        name="Cursor effects"
        onSelectedChange={() => {
          const bounds = triggerRef.current?.getBoundingClientRect();
          if (!bounds) return;
          void toggle("cursor", bounds);
        }}
        shortcut="M"
      >
        <MousePointer />
      </PreviewToolToggle>
    </span>
  );
}
