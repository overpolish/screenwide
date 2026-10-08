// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../../components/base/button/button";
import { t } from "../../../../i18n/i18n";

import type { AnnotationKind } from "../../../../components/shared/annotation-style/types";
import type { ToolPanelPatch } from "../tool-panel-patch";

/**
 * The one-shot actions at the foot of the annotation panel, in a row of their
 * own. An arrow is turned round through the same commit path the drag on the
 * picture uses, and not offered before anything is drawn; an image is
 * mirrored from its own Image row instead. Clearing belongs to the pen rather
 * than to one stroke, so it is offered before anything is drawn too; one undo
 * brings every stroke back.
 */
export function AnnotationActions({
  canClearDrawings,
  change,
  isDraft,
  isLocked,
  kind,
  reversible,
}: {
  canClearDrawings: boolean;
  change: (values: ToolPanelPatch) => void;
  isDraft: boolean;
  isLocked: boolean;
  kind: AnnotationKind;
  reversible: boolean;
}) {
  if (reversible && !isDraft && kind !== "image")
    return (
      <div className="flex justify-end">
        <Button
          aria-label={t("editor-panels-reverse-arrow")}
          isDisabled={isLocked}
          onPress={() => {
            change({ reverseAnnotation: true });
          }}
        >
          {t("editor-panels-reverse")}
        </Button>
      </div>
    );
  if (kind !== "draw") return null;
  return (
    <div className="flex justify-end">
      <Button
        aria-label={t("editor-panels-clear-drawings")}
        isDisabled={isLocked || !canClearDrawings}
        onPress={() => {
          change({ clearDrawings: true });
        }}
      >
        {t("editor-panels-clear-all")}
      </Button>
    </div>
  );
}
