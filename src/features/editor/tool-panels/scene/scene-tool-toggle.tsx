// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Clapperboard } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { ToolbarTool } from "../../../../primitives/toolbar/toolbar-tool";

/**
 * The editor toolbar's scene tool. Like the select tool it picks, moves and
 * resizes the panes, inside the scene under the playhead, and only one tool
 * is in hand at a time; its panel follows it like any canvas tool's. The
 * marker is the anchor that panel hangs from.
 */
export function SceneToolToggle({
  isSelected,
  onSelectedChange,
}: {
  isSelected: boolean;
  onSelectedChange: (selected: boolean) => void;
}) {
  return (
    <span className="inline-flex" data-editor-tool="scene">
      <ToolbarTool
        aria-label={t("editor-panels-scene")}
        isSelected={isSelected}
        label={t("editor-panels-scene")}
        onChange={onSelectedChange}
        shortcut="L"
      >
        <Clapperboard />
      </ToolbarTool>
    </span>
  );
}
