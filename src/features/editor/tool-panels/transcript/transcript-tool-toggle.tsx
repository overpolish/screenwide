// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ScrollText } from "lucide-react";
import { useRef } from "react";

import { t } from "../../../../i18n/i18n";
import { ToolbarTool } from "../../../../primitives/toolbar/toolbar-tool";
import { useToolPanel } from "../use-tool-panel";

/**
 * The editor toolbar's way into the Transcript panel, after the cursor's and
 * the keyboard's, and like them a panel with no canvas tool behind it:
 * putting it away hands the canvas back. `resetsView` is false where there
 * is no picture, so opening the panel leaves the window as it is.
 */
export function TranscriptToolToggle({
  onDismiss,
  resetsView,
}: {
  onDismiss: () => void;
  resetsView?: boolean;
}) {
  const { openTool, toggle } = useToolPanel("recording");
  const triggerRef = useRef<HTMLSpanElement>(null);

  return (
    <span
      className="inline-flex"
      data-editor-tool="transcript"
      ref={triggerRef}
    >
      <ToolbarTool
        aria-label={t("editor-panels-transcript-name")}
        isSelected={openTool === "transcript"}
        label={t("editor-panels-transcript")}
        onChange={(selected) => {
          if (!selected) {
            onDismiss();
            return;
          }
          const bounds = triggerRef.current?.getBoundingClientRect();
          if (!bounds) return;
          void toggle("transcript", bounds, resetsView);
        }}
        shortcut="Shift+T"
      >
        <ScrollText />
      </ToolbarTool>
    </span>
  );
}
