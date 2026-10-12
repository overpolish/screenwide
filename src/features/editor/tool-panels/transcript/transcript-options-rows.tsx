// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../../components/base/button/button";
import { t } from "../../../../i18n/i18n";

/** The bottom row: "Transcribe again" alongside "Restore all". Transcribe
 * again is offered beside Restore all rather than beside the first
 * Transcribe button, since it only makes sense once there is already a
 * transcript to replace. */
export function TranscriptActionsRow({
  canRestore,
  isLocked,
  onRestoreAll,
  onTranscribeAgain,
}: {
  canRestore: boolean;
  isLocked: boolean;
  onRestoreAll: () => void;
  onTranscribeAgain: () => void;
}) {
  return (
    <div className="flex justify-end gap-control">
      <Button isDisabled={isLocked} onPress={onTranscribeAgain}>
        {t("editor-panels-transcript-transcribe-again")}
      </Button>
      <Button isDisabled={isLocked || !canRestore} onPress={onRestoreAll}>
        {t("editor-panels-restore-all")}
      </Button>
    </div>
  );
}
