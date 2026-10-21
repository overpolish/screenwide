// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ControlRow } from "../../../../composites/control-row/control-row";
import { t } from "../../../../i18n/i18n";
import { Button } from "../../../../primitives/button/button";

/** Auto zoom: making the zooms that follow the cursor again, and clearing
 * them. Clear all stays in place while there is nothing to clear, so the row
 * never shifts as zooms come and go. */
export function SceneAutoZoomRow({
  hasAutoZooms,
  isDisabled,
  onClear,
  onRegenerate,
}: {
  hasAutoZooms: boolean;
  isDisabled: boolean;
  onClear: () => void;
  onRegenerate: () => void;
}) {
  return (
    <ControlRow
      controlClassName="gap-control"
      title={t("editor-panels-auto-zoom")}
    >
      {(controlProps) => (
        <>
          <Button
            {...controlProps}
            isDisabled={isDisabled}
            onPress={onRegenerate}
          >
            {t("editor-panels-regenerate")}
          </Button>
          <Button
            aria-label={t("editor-panels-clear-auto-zooms")}
            isDisabled={isDisabled || !hasAutoZooms}
            onPress={onClear}
          >
            {t("editor-panels-clear-all")}
          </Button>
        </>
      )}
    </ControlRow>
  );
}
