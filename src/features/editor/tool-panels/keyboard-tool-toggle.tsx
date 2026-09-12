// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Keyboard } from "lucide-react";
import { useRef } from "react";

import { PreviewToolToggle } from "../components/preview-tool-toggle";

import { useToolPanel } from "./use-tool-panel";

/**
 * The editor toolbar's way into the keyboard panel, next to the cursor's. A
 * second press on it puts the panel away, which is why the button's own bounds
 * travel with it.
 *
 * Keyboard is a panel and nothing else: showing it retires the canvas tool, so
 * putting it away hands the canvas back rather than leaving the editor with no
 * tool at all. The workspace says which tool that is.
 */
export function KeyboardToolToggle({ onDismiss }: { onDismiss: () => void }) {
  const { openTool, toggle } = useToolPanel("recording");
  const triggerRef = useRef<HTMLSpanElement>(null);

  return (
    <span className="inline-flex" data-editor-tool="keyboard" ref={triggerRef}>
      <PreviewToolToggle
        isSelected={openTool === "keyboard"}
        label="Keyboard"
        name="Keyboard shortcuts"
        onSelectedChange={(selected) => {
          if (!selected) {
            onDismiss();
            return;
          }
          const bounds = triggerRef.current?.getBoundingClientRect();
          if (!bounds) return;
          void toggle("keyboard", bounds);
        }}
        shortcut="K"
      >
        <Keyboard />
      </PreviewToolToggle>
    </span>
  );
}
