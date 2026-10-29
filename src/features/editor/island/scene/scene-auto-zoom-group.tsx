// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Button } from "../../../../primitives/button/button";

import { SCENE_ISLAND_WIDTH } from "./scene-width";

// The zooms planned from where the cursor clicks and the keys type: planned
// again in place of the last plan, or all taken away, which asks twice. A
// zoom changed by hand has become a scene of its own, so neither touches
// it.

export function SceneAutoZoomGroup({
  hasAutoZooms,
  onClear,
  onRegenerate,
}: {
  hasAutoZooms: boolean;
  onClear: () => void;
  onRegenerate: () => void;
}) {
  return (
    // At least the island's width, as the layout group is, and across the
    // whole section once it is stretched to the island's box. One-shot
    // actions, at the end of their row and as wide as their labels; Clear
    // zooms holds the width of its confirmation too, so arming it never
    // moves Regenerate.
    <div
      className={cn(
        "flex min-w-full justify-end gap-separate",
        SCENE_ISLAND_WIDTH.part,
      )}
    >
      <ConfirmActionButton
        confirmLabel={t("editor-island-scene-confirm-clear-auto-zooms")}
        isDisabled={!hasAutoZooms}
        label={t("editor-island-scene-clear-auto-zooms")}
        onConfirm={onClear}
      />
      <Button onPress={onRegenerate}>
        {t("editor-island-scene-regenerate-auto-zooms")}
      </Button>
    </div>
  );
}
