// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  deletedRecordingKeyboardShortcutFragments,
  deletedRecordingKeyboardShortcutIds,
  recordingKeyboardShortcutPositions,
  resetAllRecordingKeyboardShortcutPositions,
  restoreRecordingKeyboardShortcuts,
} from "../recording-keyboard-timeline-edit";
import { RecordingTimelineEdit } from "../recording-timeline-edit";
import { useExportEditGesture } from "../use-export-edit-history";

export function useRestoreRecordingKeyboardShortcuts(
  edit: RecordingTimelineEdit | null | undefined,
  onChange?: (edit: RecordingTimelineEdit) => void,
) {
  const editGesture = useExportEditGesture();
  const canRestore = Boolean(
    edit &&
    onChange &&
    (deletedRecordingKeyboardShortcutIds(edit).length > 0 ||
      deletedRecordingKeyboardShortcutFragments(edit).length > 0),
  );
  const canReset = Boolean(
    edit && onChange && recordingKeyboardShortcutPositions(edit).length > 0,
  );
  const restore = () => {
    if (!edit || !onChange) return;
    const restored = restoreRecordingKeyboardShortcuts(edit);
    if (restored === edit) return;
    editGesture.beginGesture();
    onChange(restored);
    editGesture.endGesture();
  };
  const reset = () => {
    if (!edit || !onChange) return;
    const next = resetAllRecordingKeyboardShortcutPositions(edit);
    if (next === edit) return;
    editGesture.beginGesture();
    onChange(next);
    editGesture.endGesture();
  };
  return { canRestore, reset: canReset ? reset : undefined, restore };
}
