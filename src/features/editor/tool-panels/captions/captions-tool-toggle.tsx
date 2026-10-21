// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Captions } from "lucide-react";
import { useRef } from "react";

import { t } from "../../../../i18n/i18n";
import { ToolbarTool } from "../../../../primitives/toolbar/toolbar-tool";
import { useToolPanel } from "../use-tool-panel";

/**
 * The editor toolbar's way into the Captions panel, beside the Transcript
 * tool whose words the captions are made of. Like it, a panel with no canvas
 * tool behind it: putting it away hands the canvas back. `resetsView` is
 * false where there is no picture, so opening the panel leaves the window as
 * it is.
 */
export function CaptionsToolToggle({
  onDismiss,
  resetsView,
}: {
  onDismiss: () => void;
  resetsView?: boolean;
}) {
  const { openTool, toggle } = useToolPanel("recording");
  const triggerRef = useRef<HTMLSpanElement>(null);

  return (
    <span className="inline-flex" data-editor-tool="captions" ref={triggerRef}>
      <ToolbarTool
        aria-label={t("editor-panels-captions-name")}
        isSelected={openTool === "captions"}
        label={t("editor-panels-captions")}
        onChange={(selected) => {
          if (!selected) {
            onDismiss();
            return;
          }
          const bounds = triggerRef.current?.getBoundingClientRect();
          if (!bounds) return;
          void toggle("captions", bounds, resetsView);
        }}
        shortcut="U"
      >
        <Captions />
      </ToolbarTool>
    </span>
  );
}
