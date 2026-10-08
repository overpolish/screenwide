// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../../components/base/button/button";
import { Text } from "../../../../components/base/text/text";
import { t } from "../../../../i18n/i18n";

/**
 * The panel while several annotations are chosen. They share no one dress to
 * show, so only what acts on them all is offered.
 */
export function AnnotationSelectionSummary({
  count,
  isLocked,
  onDelete,
}: {
  count: number;
  isLocked: boolean;
  onDelete: () => void;
}) {
  return (
    <div className="flex flex-col gap-section">
      <Text variant="body">
        {t("editor-panels-annotations-selected", { count })}
      </Text>
      <div className="flex justify-end">
        <Button
          aria-label={t("editor-panels-delete-annotations")}
          isDisabled={isLocked}
          onPress={onDelete}
        >
          {t("editor-panels-delete")}
        </Button>
      </div>
    </div>
  );
}
